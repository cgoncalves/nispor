// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use tokio::runtime;

use super::{
    conf::{apply_ifaces_conf, apply_ip_addrs_only, apply_routes_conf},
    query::get_iface_name2index,
};
use crate::{IfaceConf, NisporError, RouteConf};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct NetConf {
    #[serde(rename = "interfaces")]
    pub ifaces: Option<Vec<IfaceConf>>,
    pub routes: Option<Vec<RouteConf>>,
}

impl NetConf {
    pub fn apply(&self) -> Result<(), NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        rt.block_on(self.apply_async())
    }

    pub async fn apply_async(&self) -> Result<(), NisporError> {
        if let Some(ifaces) = &self.ifaces {
            apply_ifaces_conf(ifaces).await?;
        }

        if let Some(routes) = self.routes.as_ref()
            && !routes.is_empty()
        {
            let cur_iface_name_2_index = get_iface_name2index().await?;
            apply_routes_conf(routes, &cur_iface_name_2_index).await?;
        }
        Ok(())
    }

    /// Apply only IP address configurations without any link-level
    /// changes.  Unlike [`apply_async`](Self::apply_async), this does
    /// **not** send `RTM_SETLINK` messages, so it will not trigger
    /// NetworkManager to re-activate connection profiles on managed
    /// interfaces.  Routes are ignored.
    pub async fn apply_ip_addrs_only_async(&self) -> Result<(), NisporError> {
        if let Some(ifaces) = &self.ifaces {
            apply_ip_addrs_only(ifaces).await?;
        }
        Ok(())
    }
}
