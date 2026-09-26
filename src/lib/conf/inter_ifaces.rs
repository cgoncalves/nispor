// SPDX-License-Identifier: Apache-2.0

use rtnetlink::new_connection;

use super::{
    super::query::get_ifaces_with_handle, iface::apply_iface_conf,
    ip::change_ip_layer, wireguard::apply_wg_conf,
};
use crate::{
    ErrorKind, IfaceConf, IfaceType, NetStateIfaceFilter, NisporError,
};

pub(crate) async fn apply_ifaces_conf(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for des_iface in des_ifaces {
        apply_iface_conf(&handle, des_iface).await?;
    }

    let wg_ifaces: Vec<_> = des_ifaces
        .iter()
        .filter(|i| i.iface_type == Some(IfaceType::Wireguard))
        .collect();

    if !wg_ifaces.is_empty() {
        let (wg_connection, mut wg_handle, _) = nl_wireguard::new_connection()?;
        tokio::spawn(wg_connection);

        for des_iface in wg_ifaces {
            apply_wg_conf(&mut wg_handle, des_iface).await?;
        }
    }

    Ok(())
}

/// Apply only IP address changes without sending any link-level
/// (RTM_SETLINK) netlink messages.  This is useful when the caller
/// needs to add addresses to an interface managed by NetworkManager
/// without triggering NM to re-activate the connection profile.
pub(crate) async fn apply_ip_addrs_only(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for des_iface in des_ifaces {
        let mut iface_filter = NetStateIfaceFilter::minimum();
        iface_filter.iface_name = Some(des_iface.name.clone());
        iface_filter.include_ip_address = true;
        let cur_iface = if let Ok(mut ifaces) =
            get_ifaces_with_handle(&handle, Some(&iface_filter)).await
        {
            ifaces.remove(&des_iface.name)
        } else {
            None
        };
        if let Some(cur_iface) = cur_iface {
            change_ip_layer(&handle, des_iface, &cur_iface).await?;
        } else {
            return Err(NisporError::new(
                ErrorKind::Bug,
                format!(
                    "Cannot restore IP addresses: interface {} not found",
                    des_iface.name
                ),
            ));
        }
    }
    Ok(())
}
