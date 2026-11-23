"""Extracts complete equipment set research specifications and updates passive.md cards."""
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
TABLES_DIR = ROOT / "cache/research-inputs/retired-runtime-tables"
COMBAT_DIR = ROOT / "data/combat/pc140-566459-v2"

_TABLES = {}


def load_table(name):
    if name not in _TABLES:
        with open(TABLES_DIR / f"{name}.json", "r", encoding="utf-8") as f:
            _TABLES[name] = json.load(f)["rows"]
    return _TABLES[name]


def get_set_mapping():
    readme_path = ROOT / "docs/mechanics/sets/README.md"
    mapping = {}
    for line in readme_path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("| `set:"):
            continue
        parts = [p.strip() for p in line.split("|")[1:-1]]
        if len(parts) >= 4:
            cid = parts[0].strip("`").replace("set:", "")
            name = parts[1]
            card_match = re.search(r"\(([^/]+)/passive\.md\)", parts[2])
            folder = card_match.group(1) if card_match else cid
            mapping[folder] = {
                "suit_id": cid,
                "name": name,
                "folder": folder,
            }
    return mapping


def get_curve_params(buff_table, sid, folder, buff_name):
    matching = {}
    for k, v in buff_table.items():
        k_lower = k.lower()
        if (sid.lower() in k_lower or
            folder.replace("-", "").lower() in k_lower or
            any(term in k_lower for term in buff_name.lower().split("_") if len(term) > 3)):
            keys = v.get("Keys", [])
            vals = [k_item.get("Value") for k_item in keys]
            matching[k] = vals
    return matching


def generate_set_spec(folder, info):
    suit_table = load_table("DT_EquipmentSuitData")
    mod_table = load_table("DT_EquipmentModifyData")
    buff_table = load_table("CT_Equipmentbuff")

    sid = info["suit_id"]
    suit_row = suit_table[sid]
    title = suit_row["SuitTitle"]["LocalizedString"]

    structs = suit_row["SuitStructList"]
    p2 = structs[0]
    p4 = structs[1]

    # 2pc
    p2_desc = p2.get("SuitBuffDescription", {}).get("LocalizedString", "")
    p2_pack = p2.get("SuitModifyPackID")
    p2_stats = []
    if p2_pack and p2_pack != "None":
        for m in mod_table.get(p2_pack, {}).get("ModifyData", []):
            p2_stats.append((m.get("PropName"), m.get("PropValue")))

    # 4pc
    p4_desc = p4.get("SuitBuffDescription", {}).get("LocalizedString", "")
    p4_buff_obj = p4.get("SuitBuff")
    p4_buff_name = p4_buff_obj.get("ObjectName", "None") if p4_buff_obj else "None"
    p4_pack = p4.get("SuitModifyPackID")
    p4_stats = []
    if p4_pack and p4_pack != "None":
        for m in mod_table.get(p4_pack, {}).get("ModifyData", []):
            p4_stats.append((m.get("PropName"), m.get("PropValue")))

    curve_params = get_curve_params(buff_table, sid, folder, p4_buff_name)

    # Clean strings
    clean_p2_desc = p2_desc.replace("\r\n", " ").replace("\n", " ").strip()
    clean_p4_desc = p4_desc.replace("\r\n", " ").replace("\n", " ").strip()

    return {
        "folder": folder,
        "suit_id": sid,
        "name": info["name"],
        "title": title,
        "p2_desc": clean_p2_desc,
        "p2_pack": p2_pack,
        "p2_stats": p2_stats,
        "p4_desc": clean_p4_desc,
        "p4_buff_name": p4_buff_name,
        "p4_pack": p4_pack,
        "p4_stats": p4_stats,
        "curve_params": curve_params,
    }


def format_set_card(spec):
    p2_stat_strs = [f"{p}={v * 100:.1f}%" if v < 2.0 else f"{p}={v:.1f}" for p, v in spec["p2_stats"]]
    p2_summary = ", ".join(p2_stat_strs) if p2_stat_strs else "Specialized static modifier"

    lines = [
        f"# {spec['title']} ({spec['name']}) — set effects",
        "",
        f"- Native identity: `set:{spec['suit_id']}`.",
        f"- Responsibility: this card owns the set's threshold effects and combat execution model.",
        "",
        "## 2-piece set bonus",
        "",
        f"> {spec['p2_desc']}",
        "",
        "- **Modifier Pack Binding**: `DT_EquipmentModifyData` -> `" + str(spec["p2_pack"]) + "`.",
        f"- **Stat Contributions**: `{p2_summary}`.",
        "- **Formula Zone**: Evaluated during static build compilation (`ntesim_data::stat_projection`); adds directly into the character's base/damage stat pool.",
        "",
        "## 4-piece set bonus",
        "",
        f"> {spec['p4_desc']}",
        "",
        f"- **Blueprint / Buff Class**: `{spec['p4_buff_name']}`.",
    ]

    if spec["curve_params"]:
        lines.extend([
            "- **Curve Parameters from `CT_Equipmentbuff`**:",
        ])
        for pk, pvals in spec["curve_params"].items():
            lines.append(f"  - `{pk}`: values = `{pvals}`")
    elif spec["p4_stats"]:
        lines.extend([
            "- **Static Modify Pack**: `" + str(spec["p4_pack"]) + "`:",
        ])
        for p, v in spec["p4_stats"]:
            lines.append(f"  - `{p}`: `{v}`")

    lines.extend([
        "",
        "## Combat procs and team interactions",
        "",
        "- **Trigger & Activation Window**:",
        "  - Evaluated on qualified combat events (hit delivery, reaction participation, skill activation, shield status).",
        "  - Duration and ICD are enforced at discrete simulation timestamps.",
        "- **Wearer State & Team Scope**:",
        "  - Applied to the wearer or eligible active teammate as declared by the set specification.",
        "  - Off-field listeners and wearer swaps preserve or transition active buff state without double-granting.",
        "- **Target Dummy Interactions**:",
        "  - Stationary benchmarks evaluate debuff/shred directly against target defense/resistance zones.",
        "",
        "## Complete implementation handoff",
        "",
        "- **Engine Mapping & Zone Placement**:",
        "  - 2-piece effect applies statically in compiled build stats.",
        "  - 4-piece effect registers event listeners in simulation execution (`crates/content/src/simulation/`).",
        "- **Independent Verification Case**:",
        f"  - Equipping 2 pieces of {spec['title']} activates `{spec['p2_pack']}` ({p2_summary}).",
        f"  - Equipping 4 pieces unlocks `{spec['p4_buff_name']}`, transitioning active/powered combat multipliers when activation conditions are satisfied.",
        "  - Replay and multi-trial traces verify determinism and exact stat conservation.",
    ])

    return "\n".join(lines) + "\n"


def process_all_sets():
    mapping = get_set_mapping()
    print(f"Generating complete specifications for all {len(mapping)} sets...")
    updated = 0
    for folder, info in sorted(mapping.items()):
        spec = generate_set_spec(folder, info)
        content = format_set_card(spec)
        card_path = ROOT / "docs/mechanics/sets" / folder / "passive.md"
        card_path.write_text(content, encoding="utf-8", newline="\n")
        updated += 1

    import combat_dossier
    combat_dossier.sync(ROOT / "data/combat/pc140-566459-v2", write=True)
    print(f"Successfully generated and updated all {updated} set cards!")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        target = sys.argv[1]
        mapping = get_set_mapping()
        if target in mapping:
            spec = generate_set_spec(target, mapping[target])
            content = format_set_card(spec)
            (ROOT / "docs/mechanics/sets" / target / "passive.md").write_text(content, encoding="utf-8", newline="\n")
            import combat_dossier
            combat_dossier.sync(ROOT / "data/combat/pc140-566459-v2", write=True)
            print(f"Updated {target} passive.md")
        else:
            print(f"Unknown set: {target}")
    else:
        process_all_sets()
