use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::features::truck_change::parser::{
    UnitBlock, extract_array_entries, extract_field_value, is_null_ref, normalize_sii_unit_id,
    parse_unit_blocks,
};
use crate::features::truck_change::writer::set_unit_field_value;

pub const BUS_STAT_FIELDS: [&str; 8] = [
    "bus_experience_points",
    "bus_total_distance",
    "bus_finished_job_count",
    "bus_cancelled_job_count",
    "bus_total_passengers",
    "bus_total_stops",
    "bus_game_time",
    "bus_playing_time",
];

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Career {
    Truck,
    Bus,
}

impl Career {
    pub fn xp_field(self) -> &'static str {
        match self {
            Self::Truck => "experience_points",
            Self::Bus => "bus_experience_points",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusStop {
    pub id: String,
    pub resolved: bool,
    pub discovered: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusJobEntry {
    pub id: String,
    pub job_type: Option<String>,
    pub assigned_bus: Option<String>,
    pub source_stop: Option<String>,
    pub target_stop: Option<String>,
    pub stage_count: Option<i64>,
    pub exp_delta: Option<i64>,
    pub reward: Option<i64>,
    pub travelled_distance: Option<f64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub total_passengers: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusJobLog {
    pub id: String,
    pub version: Option<String>,
    pub entries: Option<String>,
    pub entries_detail: Vec<BusJobEntry>,
}

/// "bus_stop.city.helsinki" -> "Helsinki"; "bus_stop.viewpoints.pocbodo" -> "Pocbodo (viewpoint)".
pub fn display_stop_name(id: &str) -> String {
    let mut rest = id.strip_prefix("bus_stop.").unwrap_or(id);
    rest = rest.strip_prefix("city.").unwrap_or(rest);
    let viewpoint = rest.strip_prefix("viewpoints.");
    let rest = viewpoint.unwrap_or(rest);
    let mut name = title_case_words(rest);
    if viewpoint.is_some() {
        name.push_str(" (viewpoint)");
    }
    name
}

/// "vehicle.volvo.9700_23" -> "Volvo 9700 23".
pub fn display_bus_name(id: &str) -> String {
    let rest = id.strip_prefix("vehicle.").unwrap_or(id);
    title_case_words(rest)
}

/// "city.wien" -> "Wien" (job entries store stops as "city.<name>").
pub fn display_job_stop_name(raw: &str) -> String {
    let rest = raw.strip_prefix("city.").unwrap_or(raw);
    title_case_words(rest)
}

fn title_case_words(raw: &str) -> String {
    raw.split(['_', '.', '-'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachesSnapshot {
    pub save_hash: String,
    pub truck_xp: Option<i64>,
    pub bus_xp: Option<i64>,
    pub statistics: std::collections::BTreeMap<String, Option<String>>,
    pub stops: Vec<BusStop>,
    pub discovered_stops: usize,
    pub declared_stop_count: Option<String>,
    pub buses: Option<String>,
    pub bus_references: Vec<BusStop>,
    pub current_bus_job: Option<String>,
    pub current_bus_job_resolved: bool,
    pub my_vehicles_mode: Option<String>,
    pub assigned_vehicles_mode: Option<String>,
    pub job_logs: Vec<BusJobLog>,
    pub total_bus_jobs: usize,
    pub total_bus_exp: i64,
    pub total_bus_reward: i64,
    pub total_bus_distance: f64,
    pub total_bus_passengers: i64,
    pub warnings: Vec<String>,
}

pub fn save_hash(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

fn unique_unit<'a>(blocks: &'a [UnitBlock], kind: &str) -> Result<&'a UnitBlock, String> {
    let matches = blocks
        .iter()
        .filter(|block| block.unit_type == kind)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [unit] => Ok(unit),
        [] => Err(format!("coaches_unit_missing:{kind}")),
        _ => Err(format!("coaches_unit_ambiguous:{kind}")),
    }
}

fn resolve<'a>(blocks: &'a [UnitBlock], id: &str) -> Option<&'a UnitBlock> {
    let id = normalize_sii_unit_id(id);
    let mut matches = blocks
        .iter()
        .filter(|block| normalize_sii_unit_id(&block.id) == id);
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

fn xp(unit: &UnitBlock, field: &str) -> Option<i64> {
    extract_field_value(&unit.raw_block, field)?
        .parse::<i64>()
        .ok()
        .filter(|value| *value >= 0)
}

/// SII quoted strings arrive with their quotes ("city.wien"); strip one pair.
fn unquote(value: &str) -> &str {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
}

/// Truck and bus career XP from a decrypted game.sii, read straight
/// off the economy unit. Lightweight alternative to `read_snapshot`
/// for callers that only need the XP figures: the XP editors (which
/// must start from the true truck XP, not the info.sii total) and the
/// info.sii sync (which stores the total, truck + bus). Either value
/// is None when the economy unit or the field is missing.
pub fn career_xp_from_content(content: &str) -> (Option<i64>, Option<i64>) {
    let blocks = parse_unit_blocks(content);
    let Ok(economy) = unique_unit(&blocks, "economy") else {
        return (None, None);
    };
    (
        xp(economy, Career::Truck.xp_field()),
        xp(economy, Career::Bus.xp_field()),
    )
}

pub fn read_snapshot(content: &str) -> Result<CoachesSnapshot, String> {
    let blocks = parse_unit_blocks(content);
    let economy = unique_unit(&blocks, "economy")?;
    let player = match extract_field_value(&economy.raw_block, "player") {
        Some(id) => resolve(&blocks, &id).filter(|unit| unit.unit_type == "player"),
        None => unique_unit(&blocks, "player").ok(),
    };
    let player_field = |field| player.and_then(|unit| extract_field_value(&unit.raw_block, field));
    let mut warnings = Vec::new();
    let stop_entries = extract_array_entries(&economy.raw_block, "bus_stops");
    let declared_stop_count = extract_field_value(&economy.raw_block, "bus_stops");
    if declared_stop_count
        .as_deref()
        .and_then(|value| value.parse::<usize>().ok())
        != Some(stop_entries.len())
        && (declared_stop_count.is_some() || !stop_entries.is_empty())
    {
        warnings.push("coaches.stop_array_invalid".to_string());
    }
    if stop_entries
        .iter()
        .enumerate()
        .any(|(expected, (actual, _))| expected != *actual)
    {
        warnings.push("coaches.stop_array_invalid".to_string());
    }
    let mut seen = std::collections::HashSet::new();
    let stops = stop_entries
        .into_iter()
        .filter_map(|(_, id)| {
            if !seen.insert(normalize_sii_unit_id(&id)) {
                warnings.push("coaches.stop_array_invalid".to_string());
                return None;
            }
            let stop = resolve(&blocks, &id).filter(|unit| unit.unit_type == "bus_stop");
            let discovered = stop
                .and_then(|unit| extract_field_value(&unit.raw_block, "discovered"))
                .and_then(|value| match value.as_str() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                });
            if stop.is_none() || discovered.is_none() {
                warnings.push("coaches.stop_unresolved".to_string());
            }
            Some(BusStop {
                id,
                resolved: stop.is_some(),
                discovered,
            })
        })
        .collect::<Vec<_>>();
    let buses = player_field("buses");
    let bus_entries = player
        .map(|unit| extract_array_entries(&unit.raw_block, "buses"))
        .unwrap_or_default();
    if buses
        .as_deref()
        .and_then(|value| value.parse::<usize>().ok())
        != Some(bus_entries.len())
        && (buses.is_some() || !bus_entries.is_empty())
    {
        warnings.push("coaches.bus_array_invalid".to_string());
    }
    let bus_references = bus_entries
        .into_iter()
        .map(|(_, id)| {
            let resolved = resolve(&blocks, &id).is_some();
            BusStop {
                id,
                resolved,
                discovered: None,
            }
        })
        .collect();
    let current_bus_job = player_field("current_bus_job");
    let current_bus_job_resolved = current_bus_job
        .as_deref()
        .filter(|value| !is_null_ref(value))
        .and_then(|id| resolve(&blocks, id))
        .is_some();
    let job_logs = blocks
        .iter()
        .filter(|unit| unit.unit_type == "bus_job_log")
        .map(|unit| {
            let entries_detail = extract_array_entries(&unit.raw_block, "entries")
                .into_iter()
                .filter_map(|(_, id)| {
                    let entry = resolve(&blocks, &id)?;
                    if entry.unit_type != "bus_job_log_entry" {
                        return None;
                    }
                    let field = |name: &str| extract_field_value(&entry.raw_block, name);
                    let text = |name: &str| field(name).map(|value| unquote(&value).to_string());
                    Some(BusJobEntry {
                        id,
                        job_type: text("type"),
                        assigned_bus: text("assigned_bus"),
                        source_stop: text("source_bus_stop_name"),
                        target_stop: text("target_bus_stop_name"),
                        stage_count: field("stage_count").and_then(|v| v.parse().ok()),
                        exp_delta: field("exp_delta").and_then(|v| v.parse().ok()),
                        reward: field("reward").and_then(|v| v.parse().ok()),
                        travelled_distance: field("travelled_distance")
                            .and_then(|v| v.parse().ok()),
                        start_time: field("start_time").and_then(|v| v.parse().ok()),
                        end_time: field("end_time").and_then(|v| v.parse().ok()),
                        total_passengers: field("total_passengers").and_then(|v| v.parse().ok()),
                    })
                })
                .collect::<Vec<_>>();
            BusJobLog {
                id: unit.id.clone(),
                version: extract_field_value(&unit.raw_block, "version"),
                entries: extract_field_value(&unit.raw_block, "entries"),
                entries_detail,
            }
        })
        .collect::<Vec<_>>();
    let all_entries = job_logs
        .iter()
        .flat_map(|log| log.entries_detail.iter())
        .collect::<Vec<_>>();
    let total_bus_jobs = all_entries.len();
    let total_bus_exp = all_entries.iter().filter_map(|e| e.exp_delta).sum();
    let total_bus_reward = all_entries.iter().filter_map(|e| e.reward).sum();
    let total_bus_distance = all_entries
        .iter()
        .filter_map(|e| e.travelled_distance)
        .sum();
    let total_bus_passengers = all_entries.iter().filter_map(|e| e.total_passengers).sum();
    warnings.sort();
    warnings.dedup();
    Ok(CoachesSnapshot {
        save_hash: save_hash(content),
        truck_xp: xp(economy, Career::Truck.xp_field()),
        bus_xp: xp(economy, Career::Bus.xp_field()),
        statistics: BUS_STAT_FIELDS
            .iter()
            .map(|field| {
                (
                    field.to_string(),
                    extract_field_value(&economy.raw_block, field),
                )
            })
            .collect(),
        discovered_stops: stops
            .iter()
            .filter(|stop| stop.discovered == Some(true))
            .count(),
        declared_stop_count,
        stops,
        buses,
        bus_references,
        current_bus_job,
        current_bus_job_resolved,
        my_vehicles_mode: player_field("my_vehicles_mode"),
        assigned_vehicles_mode: player_field("assigned_vehicles_mode"),
        job_logs,
        total_bus_jobs,
        total_bus_exp,
        total_bus_reward,
        total_bus_distance,
        total_bus_passengers,
        warnings,
    })
}

/// Uses the existing SII unit parser/writer; never reconstructs a save from a DTO.
pub fn patch_xp(content: &str, career: Career, value: i64) -> Result<String, String> {
    if !(0..=9_007_199_254_740_991).contains(&value) {
        return Err("coaches.xp_invalid".to_string());
    }
    let blocks = parse_unit_blocks(content);
    let economy = unique_unit(&blocks, "economy")?;
    if resolve(&blocks, &economy.id).is_none() {
        return Err("coaches_unit_ambiguous:economy".to_string());
    }
    let field = career.xp_field();
    if xp(economy, field).is_none() {
        return Err("coaches.xp_missing".to_string());
    }
    let occurrences = economy
        .raw_block
        .lines()
        .filter(|line| line.trim_start().starts_with(&format!("{field}:")))
        .count();
    if occurrences != 1 {
        return Err("coaches.xp_invalid".to_string());
    }
    let (mut updated, changed) =
        set_unit_field_value(content, &economy.id, field, &value.to_string())?;
    if !changed {
        return Err("coaches.xp_missing".to_string());
    }
    // Preserve the original CRLF convention after the existing line-based writer.
    if content.contains("\r\n") && !content.replace("\r\n", "").contains('\n') {
        updated = updated.replace('\n', "\r\n");
    }
    verify_xp(&updated, career, value)?;
    Ok(updated)
}

pub fn verify_xp(content: &str, career: Career, expected: i64) -> Result<(), String> {
    let blocks = parse_unit_blocks(content);
    let economy = unique_unit(&blocks, "economy")?;
    if xp(economy, career.xp_field()) != Some(expected) {
        return Err("coaches.verify_failed".to_string());
    }
    Ok(())
}
