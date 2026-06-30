pub struct EmbeddedPet {
    pub id: &'static str,
    pub manifest_name: &'static str,
    pub manifest: &'static [u8],
    pub spritesheet_name: &'static str,
    pub spritesheet: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/embedded_pets.rs"));
