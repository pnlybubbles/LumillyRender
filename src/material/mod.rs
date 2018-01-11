mod traits;
mod lambert;
mod phong;
mod blinn_phong;

pub mod material {
  pub use super::traits::*;
  pub use super::lambert::*;
  pub use super::phong::*;
  pub use super::blinn_phong::*;
}
