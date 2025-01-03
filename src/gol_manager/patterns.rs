#![cfg_attr(rustfmt, rustfmt_skip)]

use std::sync::OnceLock;

pub struct Pattern {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn get_blinker() -> &'static Pattern {
    static BLINKER: OnceLock<Pattern> = OnceLock::new();
    BLINKER.get_or_init(|| Pattern {
        data: vec![0, 1, 0, 0, 1, 0, 0, 1, 0],
        width: 3,
        height: 3,
    })
}

pub fn get_loaf() -> &'static Pattern {
    static LOAF: OnceLock<Pattern> = OnceLock::new();
    LOAF.get_or_init(|| Pattern {
        data: vec![
            0, 1, 1, 0,
            1, 0, 0, 1,
            0, 1, 0, 1,
            0, 0, 1, 0
        ],
        width: 4,
        height: 4,
    })
}

pub fn get_toad() -> &'static Pattern {
    static TOAD: OnceLock<Pattern> = OnceLock::new();
    TOAD.get_or_init(|| Pattern {
        data: vec![
            0, 0, 1, 0,
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 0, 0
        ],
        width: 4,
        height: 4,
    })
}

pub fn get_light_weight_spaceship() -> &'static Pattern {
    static LIGHT_WEIGHT_SPACESHIP: OnceLock<Pattern> = OnceLock::new();
    LIGHT_WEIGHT_SPACESHIP.get_or_init(|| Pattern {
        data: vec![
            0, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            1, 0, 0, 1, 0
        ],
        width: 5,
        height: 4,
    })
}

pub fn get_middle_weight_spaceship() -> &'static Pattern {
    static MIDDLE_WEIGHT_SPACESHIP: OnceLock<Pattern> = OnceLock::new();
    MIDDLE_WEIGHT_SPACESHIP.get_or_init(|| Pattern {
        data: vec![
            0, 0, 1, 0, 0, 0,
            1, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 1,
            0, 1, 1, 1, 1, 1,
        ],
        width: 6,
        height: 5,
    })
}

pub fn get_heavy_weight_spaceship() -> &'static Pattern {
    static HEAVY_WEIGHT_SPACESHIP: OnceLock<Pattern> = OnceLock::new();
    HEAVY_WEIGHT_SPACESHIP.get_or_init(|| Pattern {
        data: vec![
            0, 0, 1, 1, 0, 0, 0,
            1, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 1,
            0, 1, 1, 1, 1, 1, 1,
        ],
        width: 7,
        height: 5,
    })
}

pub fn get_penta_decathlon() -> &'static Pattern {
    static PENTA_DECATHLON: OnceLock<Pattern> = OnceLock::new();
    PENTA_DECATHLON.get_or_init(|| Pattern {
        data: vec![
            0, 0, 0, 1, 1, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 1, 0, 0,
            0, 1, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 1, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 1, 1, 1, 0, 0, 0,
        ],
        width: 9,
        height: 10,
    })
}
