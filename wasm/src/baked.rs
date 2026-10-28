// What bake.py made from the game's assets (wasm/baked/, generated)

pub struct Baked {
    pub png: &'static [u8],
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/baked/baked.rs"));
