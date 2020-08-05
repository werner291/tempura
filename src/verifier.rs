
use crate::build::fragment_builder::{FragmentBuilder};
use crate::program::LacunaryRef;
use crate::program::Operation;

enum VerifyResult {
    True,
    False,
    Unsure
}

pub fn verify_claim(env: &FragmentBuilder, lr: &LacunaryRef, claim: &Operation<LacunaryRef>) -> bool {

    

    unimplemented!()

}