use crate::simulation::{
    ExactFraction as F, GameScenarioRequest, GearProcConfig, state::GameSimulatorState,
};

pub(crate) fn bonus(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    recipient: usize,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<F, String> {
    let element = |index: usize| {
        crate::registry::get_character_descriptor_by_id(&request.actors[index].character_id)
            .map(|d| d.element.clone())
    };
    let recipient_element = element(recipient)?;
    let mut own = F::new(0, 1)?;
    for proc in &request.actors[recipient].gear_procs {
        let (amount, id) = match proc {
            GearProcConfig::BlackBookBreak {
                break_intensity_bonus,
            } => (
                *break_intensity_bonus,
                crate::weapons::black_book::HYPOTHESIS_ID,
            ),
            GearProcConfig::JiaojuanBreak {
                break_intensity_bonus,
                ..
            } => (
                *break_intensity_bonus,
                crate::weapons::jiaojuan::HYPOTHESIS_ID,
            ),
            _ => continue,
        };
        crate::runtime_api::record_hypothesis_id(request, assumptions, id)?;
        own = own.add(F::from_ratio(amount)?)?;
    }
    let matching = (0..request.actors.len())
        .map(element)
        .collect::<Result<Vec<_>, _>>()?;
    if matching
        .iter()
        .filter(|element| **element == recipient_element)
        .count()
        < 3
    {
        return Ok(own);
    }
    let mut aura = F::new(0, 1)?;
    for (index, config) in request.actors.iter().enumerate() {
        if state.actors[index].hp == 0 || matching[index] != recipient_element {
            continue;
        }
        for proc in &config.gear_procs {
            let GearProcConfig::TuansanlangTeam {
                break_intensity_bonus,
            } = proc
            else {
                continue;
            };
            crate::runtime_api::record_hypothesis_id(
                request,
                assumptions,
                crate::weapons::tuansanlang::HYPOTHESIS_ID,
            )?;
            let candidate = F::from_ratio(*break_intensity_bonus)?;
            let lhs = candidate
                .numerator
                .checked_mul(aura.denominator)
                .ok_or("Break aura comparison overflow")?;
            let rhs = aura
                .numerator
                .checked_mul(candidate.denominator)
                .ok_or("Break aura comparison overflow")?;
            if lhs > rhs {
                aura = candidate;
            }
        }
    }
    own.add(aura)
}
