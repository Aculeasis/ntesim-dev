use crate::simulation::GameInitialResources;

pub(crate) fn validate_initial_resources(
    character: &str,
    level: u32,
    awakening: &ntesim_data::AwakeningSelection,
    initial: &GameInitialResources,
) -> Result<(), String> {
    if initial.timepiece.is_some() {
        if character != "character:1052" {
            return Err("start_timepiece is only supported by Hotori.".into());
        }
        crate::characters::hotori::timepiece::initial_pool(initial.timepiece)?;
    }
    if initial.grains.is_some() || initial.loans.is_some() {
        if character != "character:1073" {
            return Err("start_grains/start_loans are only supported by Chiz.".into());
        }
        for value in [initial.grains, initial.loans].into_iter().flatten() {
            crate::characters::chiz::resource::initial_pool(level, awakening, Some(value))?;
        }
    }
    if initial.malice.is_some() {
        if character != "character:1042" {
            return Err("start_malice is only supported by Blackbird.".into());
        }
        crate::characters::blackbird::malice::initial_pool(initial.malice)?;
    }
    if initial.crime.is_some() {
        if character != "character:1071" {
            return Err("start_crime is only supported by Chaos.".into());
        }
        crate::characters::chaos::resources::initial_pool(initial.crime)?;
    }
    if initial.power.is_some() {
        if character != "character:1025" {
            return Err("start_power is only supported by Hathor.".into());
        }
        crate::characters::hathor::power::initial_pool(initial.power)?;
    }
    if initial.moon.is_some() {
        if character != "character:1075" {
            return Err("start_moon is only supported by Iroi.".into());
        }
        crate::characters::iroi::resource::initial_value(initial.moon)?;
    }
    if initial.plot.is_some() {
        if character != "character:1020" {
            return Err("start_plot is only supported by Haniel.".into());
        }
        crate::characters::haniel::resource::initial_value(initial.plot, awakening)?;
    }
    if initial.bullets.is_some() {
        if character != "character:1055" {
            return Err("start_bullets is only supported by Jiuyuan.".into());
        }
        crate::characters::jiuyuan::resource::initial_value(initial.bullets)?;
    }
    if initial.resonance.is_some() {
        if character != "character:1054" {
            return Err("start_resonance is only supported by Daffodil.".into());
        }
        crate::characters::daffodil::resonance::initial_value(initial.resonance)?;
    }
    if initial.karma.is_some() {
        if character != "character:1033" {
            return Err("start_karma is only supported by Adler.".into());
        }
        crate::characters::adler::awakenings::initial_karma(initial.karma)?;
    }
    if initial.keys.is_some() {
        if character != "character:1021" {
            return Err("start_keys is only supported by Edgar.".into());
        }
        crate::characters::edgar::awakenings::initial_keys(initial.keys)?;
    }
    if initial.stocks.is_some() {
        if character != "character:1073" {
            return Err("start_stocks/start_skill_stocks is only supported by Chiz.".into());
        }
        crate::characters::chiz::pressed::initial_stocks(initial.stocks)?;
    }
    if initial.kindling.is_some() {
        if character != "character:1036" {
            return Err("start_kindling is only supported by Zankou.".into());
        }
        crate::characters::zankou::passives::initial_kindling(initial.kindling)?;
    }
    Ok(())
}

pub(crate) fn initial_resource_assumptions(
    initial: &GameInitialResources,
) -> std::collections::BTreeMap<String, String> {
    let mut values = if initial.grains.is_some() || initial.loans.is_some() {
        crate::characters::chiz::financial_source_assumptions()
    } else {
        Default::default()
    };
    if initial.malice.is_some() {
        let id = crate::characters::blackbird::malice::MODEL;
        values.insert(id.into(), id.description().into());
    }
    if initial.crime.is_some() {
        let id = crate::characters::chaos::resources::MODEL;
        values.insert(id.into(), id.description().into());
    }
    if initial.power.is_some() {
        // Power enables the native clock; A3 additionally needs live Burst state.
        for id in [
            crate::characters::hathor::power::MODEL,
            crate::characters::hathor::burst::MODEL,
        ] {
            values.insert(id.into(), id.description().into());
        }
    }
    if initial.moon.is_some() {
        let id = crate::characters::iroi::dream::HYPOTHESIS_ID;
        values.insert(id.into(), id.description().into());
    }
    if initial.karma.is_some() {
        let id = crate::characters::adler::skill::KARMA_HYPOTHESIS_ID;
        values.insert(id.into(), id.description().into());
    }
    if initial.keys.is_some() {
        let id = crate::characters::edgar::skill::KEY_HYPOTHESIS_ID;
        values.insert(id.into(), id.description().into());
    }
    if initial.stocks.is_some() {
        for (id, desc) in crate::characters::chiz::financial_source_assumptions() {
            values.insert(id, desc);
        }
    }
    if initial.kindling.is_some() {
        let id = crate::characters::zankou::passives::HYPOTHESIS_ID;
        values.insert(id.into(), id.description().into());
    }
    values
}
