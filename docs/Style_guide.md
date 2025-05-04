#![cfg(test)] // 1. **Outer** attributes that affect the whole crate/file

//! Top‑level documentation explaining what the module does
//! More details… // 2. Inner docs (`//!`) come next

use std::fmt::{self, Display};
use std::collections::HashMap; // 3 a. Std‑lib imports (alphabetical)

use anyhow::Result;
use serde::{Deserialize, Serialize}; // 3 b. External‑crate imports

use crate::utils::slugify; // 3 c. `crate`, `super`, `self` imports

pub mod parsing; // 4. Public sub‑modules (if any)

pub struct Article { … } // 5. Definitions
