//! This module provides functions and structures about user info

use crate::{bapi_def, ApiMap};
use lazy_static::lazy_static;

// Sub-mod
mod my_info;
mod nav_info;
mod vip_info;

lazy_static! {
    static ref USER_APIS: ApiMap = bapi_def!("user.json");
}
