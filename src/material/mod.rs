mod traits;
mod lambert;
mod phong;
mod blinn_phong;
mod ggx;

pub mod material {
  pub use super::traits::*;
  pub use super::lambert::*;
  pub use super::phong::*;
  pub use super::blinn_phong::*;
  pub use super::ggx::*;
}
