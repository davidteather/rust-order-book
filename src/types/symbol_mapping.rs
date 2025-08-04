use phf::phf_map;

pub type SymbolId = u16;

pub static SYMBOL_TO_ID: phf::Map<&'static str, SymbolId> = phf_map! {
    "AAPL" => 0,
    "GOOGL" => 1,
    "TSLA" => 2,
};