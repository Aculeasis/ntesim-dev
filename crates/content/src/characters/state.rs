#[derive(Clone, Debug)]
pub(crate) enum RuntimeState {
    Generic,
    Mint(super::mint::skill::State),
    Sakiri(super::sakiri::awakenings::State),
    Zero(super::zero::awakenings::State),
    Iroi(super::iroi::awakenings::State),
    Daffodil(super::daffodil::awakenings::State),
    Nanally(super::nanally::awakenings::State),
    Hotori(super::hotori::awakenings::State),
    Edgar(super::edgar::awakenings::State),
    Blackbird(Box<super::blackbird::skill::State>),
    Skia(super::skia::skill::State),
    Haniel(super::haniel::skill::State),
    Fadia(super::fadia::skill::State),
    Lacrimosa(super::lacrimosa::skill::State),
    Zankou(super::zankou::awakenings::State),
    Hathor(super::hathor::awakenings::State),
    Shinku(super::shinku::awakenings::State),
    Baicang(super::baicang::awakenings::State),
    Aurelia(super::aurelia::awakenings::State),
    Chiz(super::chiz::awakenings::State),
    Jiuyuan(super::jiuyuan::skill::State),
    Chaos(Box<super::chaos::skill::State>),
    Linko(super::linko::awakenings::State),
    Adler(super::adler::awakenings::State),
    Akane(Box<super::akane::skill::State>),
}
impl RuntimeState {
    pub(crate) fn new(character_id: &str) -> Self {
        match character_id {
            "character:1019" => Self::Mint(Default::default()),
            "character:1003" => Self::Sakiri(Default::default()),
            "character:1046" => Self::Zero(Default::default()),
            "character:1075" => Self::Iroi(Default::default()),
            "character:1054" => Self::Daffodil(Default::default()),
            "character:1010" => Self::Nanally(Default::default()),
            "character:1052" => Self::Hotori(Default::default()),
            "character:1021" => Self::Edgar(Default::default()),
            "character:1042" => Self::Blackbird(Default::default()),
            "character:1008" => Self::Skia(Default::default()),
            "character:1020" => Self::Haniel(Default::default()),
            "character:1039" => Self::Fadia(Default::default()),
            "character:1004" => Self::Lacrimosa(Default::default()),
            "character:1036" => Self::Zankou(Default::default()),
            "character:1025" => Self::Hathor(Default::default()),
            "character:1076" => Self::Shinku(Default::default()),
            "character:1023" => Self::Baicang(Default::default()),
            "character:1070" => Self::Aurelia(Default::default()),
            "character:1073" => Self::Chiz(Default::default()),
            "character:1055" => Self::Jiuyuan(Default::default()),
            "character:1071" => Self::Chaos(Default::default()),
            "character:1072" => Self::Linko(Default::default()),
            "character:1033" => Self::Adler(Default::default()),
            "character:1057" => Self::Akane(Default::default()),
            _ => Self::Generic,
        }
    }
    /// Ordinary combo fields only; a replayed virtual cast restores these
    /// without reverting source effects it legitimately re-runs.
    pub(crate) fn normal_chain_fields(&mut self) -> Option<(&mut usize, &mut Option<u64>)> {
        match self {
            Self::Generic => None,
            Self::Mint(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Sakiri(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Zero(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Iroi(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Daffodil(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Nanally(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Hotori(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Edgar(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Blackbird(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Skia(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Haniel(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Fadia(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Lacrimosa(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Zankou(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Hathor(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Shinku(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Baicang(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Aurelia(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Chiz(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Jiuyuan(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Chaos(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Linko(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Adler(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
            Self::Akane(state) => Some((&mut state.normal_stage, &mut state.normal_started_us)),
        }
    }
    pub(crate) fn reset_normal_chain(&mut self) {
        match self {
            Self::Generic => {}
            Self::Mint(s) => {
                s.riposte_ready = false;
                s.airborne = false;
                s.entry_normal_pending = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Sakiri(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Zero(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Iroi(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Daffodil(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Nanally(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Hotori(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Edgar(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Blackbird(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Skia(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
                s.gnaw_until_us = 0;
                s.gnaw_generation = s.gnaw_generation.saturating_add(1);
            }
            Self::Haniel(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Fadia(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Lacrimosa(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Zankou(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Hathor(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Shinku(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Baicang(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
                s.heavy_dodge_until_us = 0;
                s.cancel_window = None;
                s.clock.end_action();
            }
            Self::Aurelia(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Chiz(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Jiuyuan(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Chaos(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Linko(s) => {
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Adler(s) => {
                s.riposte_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
            Self::Akane(s) => {
                s.riposte_ready = false;
                s.melee_pre_ready = false;
                s.entry_normal_ready = false;
                s.normal_stage = 0;
                s.normal_started_us = None;
            }
        }
    }
    pub(crate) fn mint_mut(&mut self) -> Result<&mut super::mint::skill::State, String> {
        match self {
            Self::Mint(state) => Ok(state),
            _ => Err("Mint runtime extension unavailable".into()),
        }
    }
    pub(crate) fn sakiri_mut(&mut self) -> Result<&mut super::sakiri::awakenings::State, String> {
        match self {
            Self::Sakiri(state) => Ok(state),
            _ => Err("Sakiri runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn zero_mut(&mut self) -> Result<&mut super::zero::awakenings::State, String> {
        match self {
            Self::Zero(state) => Ok(state),
            _ => Err("Zero runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn iroi(&self) -> Result<&super::iroi::awakenings::State, String> {
        match self {
            Self::Iroi(state) => Ok(state),
            _ => Err("Iroi runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn iroi_mut(&mut self) -> Result<&mut super::iroi::awakenings::State, String> {
        match self {
            Self::Iroi(state) => Ok(state),
            _ => Err("Iroi runtime extension unavailable".into()),
        }
    }
    pub(crate) fn daffodil_mut(
        &mut self,
    ) -> Result<&mut super::daffodil::awakenings::State, String> {
        match self {
            Self::Daffodil(state) => Ok(state),
            _ => Err("Daffodil runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn nanally(&self) -> Result<&super::nanally::awakenings::State, String> {
        match self {
            Self::Nanally(state) => Ok(state),
            _ => Err("Nanally runtime extension unavailable".into()),
        }
    }
    pub(crate) fn nanally_mut(&mut self) -> Result<&mut super::nanally::awakenings::State, String> {
        match self {
            Self::Nanally(state) => Ok(state),
            _ => Err("Nanally runtime extension unavailable".into()),
        }
    }
    pub(crate) fn hotori_mut(&mut self) -> Result<&mut super::hotori::awakenings::State, String> {
        match self {
            Self::Hotori(state) => Ok(state),
            _ => Err("Hotori runtime extension unavailable".into()),
        }
    }
    pub(crate) fn edgar_mut(&mut self) -> Result<&mut super::edgar::awakenings::State, String> {
        match self {
            Self::Edgar(state) => Ok(state),
            _ => Err("Edgar runtime extension unavailable".into()),
        }
    }
    pub(crate) fn haniel_mut(&mut self) -> Result<&mut super::haniel::skill::State, String> {
        match self {
            Self::Haniel(state) => Ok(state),
            _ => Err("Haniel runtime extension unavailable".into()),
        }
    }
    pub(crate) fn fadia_mut(&mut self) -> Result<&mut super::fadia::skill::State, String> {
        match self {
            Self::Fadia(state) => Ok(state),
            _ => Err("Fadia runtime extension unavailable".into()),
        }
    }
    pub(crate) fn blackbird(&self) -> Result<&super::blackbird::skill::State, String> {
        match self {
            Self::Blackbird(state) => Ok(state),
            _ => Err("Blackbird runtime extension unavailable".into()),
        }
    }
    pub(crate) fn blackbird_mut(&mut self) -> Result<&mut super::blackbird::skill::State, String> {
        match self {
            Self::Blackbird(state) => Ok(state),
            _ => Err("Blackbird runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn skia(&self) -> Result<&super::skia::skill::State, String> {
        match self {
            Self::Skia(state) => Ok(state),
            _ => Err("Skia runtime extension unavailable".into()),
        }
    }
    pub(crate) fn skia_mut(&mut self) -> Result<&mut super::skia::skill::State, String> {
        match self {
            Self::Skia(state) => Ok(state),
            _ => Err("Skia runtime extension unavailable".into()),
        }
    }
    pub(crate) fn lacrimosa_mut(&mut self) -> Result<&mut super::lacrimosa::skill::State, String> {
        match self {
            Self::Lacrimosa(state) => Ok(state),
            _ => Err("Lacrimosa runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn zankou(&self) -> Result<&super::zankou::awakenings::State, String> {
        match self {
            Self::Zankou(state) => Ok(state),
            _ => Err("Zankou runtime extension unavailable".into()),
        }
    }
    pub(crate) fn zankou_mut(&mut self) -> Result<&mut super::zankou::awakenings::State, String> {
        match self {
            Self::Zankou(state) => Ok(state),
            _ => Err("Zankou runtime extension unavailable".into()),
        }
    }
    pub(crate) fn hathor_mut(&mut self) -> Result<&mut super::hathor::awakenings::State, String> {
        match self {
            Self::Hathor(state) => Ok(state),
            _ => Err("Hathor runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn shinku(&self) -> Result<&super::shinku::awakenings::State, String> {
        match self {
            Self::Shinku(state) => Ok(state),
            _ => Err("Shinku runtime extension unavailable".into()),
        }
    }
    pub(crate) fn shinku_mut(&mut self) -> Result<&mut super::shinku::awakenings::State, String> {
        match self {
            Self::Shinku(state) => Ok(state),
            _ => Err("Shinku runtime extension unavailable".into()),
        }
    }
    pub(crate) fn baicang_mut(&mut self) -> Result<&mut super::baicang::awakenings::State, String> {
        match self {
            Self::Baicang(state) => Ok(state),
            _ => Err("Baicang runtime extension unavailable".into()),
        }
    }
    pub(crate) fn aurelia(&self) -> Result<&super::aurelia::awakenings::State, String> {
        match self {
            Self::Aurelia(state) => Ok(state),
            _ => Err("Aurelia runtime extension unavailable".into()),
        }
    }
    pub(crate) fn aurelia_mut(&mut self) -> Result<&mut super::aurelia::awakenings::State, String> {
        match self {
            Self::Aurelia(state) => Ok(state),
            _ => Err("Aurelia runtime extension unavailable".into()),
        }
    }
    pub(crate) fn chiz_mut(&mut self) -> Result<&mut super::chiz::awakenings::State, String> {
        match self {
            Self::Chiz(state) => Ok(state),
            _ => Err("Chiz runtime extension unavailable".into()),
        }
    }
    pub(crate) fn jiuyuan_mut(&mut self) -> Result<&mut super::jiuyuan::skill::State, String> {
        match self {
            Self::Jiuyuan(state) => Ok(state),
            _ => Err("Jiuyuan runtime extension unavailable".into()),
        }
    }
    pub(crate) fn chaos_mut(&mut self) -> Result<&mut super::chaos::skill::State, String> {
        match self {
            Self::Chaos(state) => Ok(state),
            _ => Err("Chaos runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn linko(&self) -> Result<&super::linko::awakenings::State, String> {
        match self {
            Self::Linko(state) => Ok(state),
            _ => Err("Linko runtime extension unavailable".into()),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn linko_mut(&mut self) -> Result<&mut super::linko::awakenings::State, String> {
        match self {
            Self::Linko(state) => Ok(state),
            _ => Err("Linko runtime extension unavailable".into()),
        }
    }
    pub(crate) fn adler_mut(&mut self) -> Result<&mut super::adler::awakenings::State, String> {
        match self {
            Self::Adler(state) => Ok(state),
            _ => Err("Adler runtime extension unavailable".into()),
        }
    }
    pub(crate) fn akane(&self) -> Result<&super::akane::skill::State, String> {
        match self {
            Self::Akane(state) => Ok(state),
            _ => Err("Akane runtime extension unavailable".into()),
        }
    }
    pub(crate) fn akane_mut(&mut self) -> Result<&mut super::akane::skill::State, String> {
        match self {
            Self::Akane(state) => Ok(state),
            _ => Err("Akane runtime extension unavailable".into()),
        }
    }
}
