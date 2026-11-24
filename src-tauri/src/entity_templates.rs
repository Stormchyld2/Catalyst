//! File-based entity/template registry for Story Development.
//!
//! Built-in templates and user-authored Markdown templates share one registry.
//! User templates are discovered recursively from the local Templates folder;
//! no database is involved.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTemplateField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTemplateManifest {
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(default = "default_template_scope")]
    pub scope: String,
    pub document_type: String,
    #[serde(default)]
    pub subtype: Option<String>,
    pub version: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub overview_fields: Vec<String>,
    #[serde(default = "default_image_field")]
    pub image_field: String,
    #[serde(default)]
    pub fields: Vec<EntityTemplateField>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntityTemplate {
    #[serde(flatten)]
    pub manifest: EntityTemplateManifest,
    pub body: String,
    pub source: String,
    pub relative_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntityTemplateIssue {
    pub relative_path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct EntityTemplateRegistry {
    pub templates: Vec<EntityTemplate>,
    pub issues: Vec<EntityTemplateIssue>,
    pub user_templates_dir: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveUserEntityTemplateInput {
    pub manifest: EntityTemplateManifest,
    pub body: String,
    pub relative_path: String,
    #[serde(default)]
    pub previous_relative_path: Option<String>,
}

fn default_image_field() -> String {
    "image".into()
}

fn default_template_scope() -> String {
    "story-development".into()
}

pub fn load_registry(user_templates_dir: &Path) -> Result<EntityTemplateRegistry, String> {
    if !user_templates_dir.is_absolute() {
        return Err("Templates folder must be an absolute local path.".into());
    }

    fs::create_dir_all(user_templates_dir).map_err(|e| {
        format!(
            "Could not create Templates folder '{}': {e}",
            user_templates_dir.display()
        )
    })?;

    let mut registry = EntityTemplateRegistry {
        templates: built_in_templates(),
        issues: Vec::new(),
        user_templates_dir: user_templates_dir.to_string_lossy().into_owned(),
    };

    let mut claimed_ids = registry
        .templates
        .iter()
        .map(|template| template.manifest.id.clone())
        .collect::<BTreeSet<_>>();

    let mut paths = Vec::new();
    collect_markdown(user_templates_dir, user_templates_dir, &mut paths)?;
    paths.sort();

    for relative_path in paths {
        let absolute_path = user_templates_dir.join(&relative_path);
        let display_path = relative_path.to_string_lossy().replace('\\', "/");
        match read_user_template(&absolute_path, &relative_path) {
            Ok(template) => {
                let id = template.manifest.id.clone();
                if !claimed_ids.insert(id.clone()) {
                    registry.issues.push(EntityTemplateIssue {
                        relative_path: display_path,
                        message: format!(
                            "Template id '{id}' is already registered. Duplicate templates are not loaded."
                        ),
                    });
                } else {
                    registry.templates.push(template);
                }
            }
            Err(message) => registry.issues.push(EntityTemplateIssue {
                relative_path: display_path,
                message,
            }),
        }
    }

    registry.templates.sort_by(|a, b| {
        a.manifest
            .category
            .cmp(&b.manifest.category)
            .then_with(|| a.manifest.name.to_lowercase().cmp(&b.manifest.name.to_lowercase()))
            .then_with(|| a.manifest.id.cmp(&b.manifest.id))
    });

    Ok(registry)
}

fn collect_markdown(root: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|e| format!("Could not read Templates folder '{}': {e}", directory.display()))?
    {
        let entry = entry.map_err(|e| format!("Could not inspect Templates folder entry: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Could not inspect template '{}': {e}", path.display()))?;

        if file_type.is_dir() {
            collect_markdown(root, &path, output)?;
        } else if file_type.is_file() && path.extension().and_then(|v| v.to_str()) == Some("md") {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "Template path escaped the Templates folder.".to_string())?;
            output.push(relative.to_path_buf());
        }
    }
    Ok(())
}

fn read_user_template(path: &Path, relative_path: &Path) -> Result<EntityTemplate, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Could not read template '{}': {e}", path.display()))?;
    let (yaml, body) = split_frontmatter(&raw)?;
    let root: BTreeMap<String, serde_yaml::Value> =
        serde_yaml::from_str(yaml).map_err(|e| format!("Invalid template YAML: {e}"))?;
    let value = root
        .get("lithomark_template")
        .ok_or_else(|| "Template frontmatter is missing 'lithomark_template'.".to_string())?
        .clone();
    let manifest: EntityTemplateManifest = serde_yaml::from_value(value)
        .map_err(|e| format!("Invalid lithomark_template metadata: {e}"))?;
    validate_manifest(&manifest)?;

    Ok(EntityTemplate {
        manifest,
        body: normalize_body(body),
        source: "user".into(),
        relative_path: Some(relative_path.to_string_lossy().replace('\\', "/")),
    })
}

pub fn validate_manifest(manifest: &EntityTemplateManifest) -> Result<(), String> {
    if manifest.id.trim().is_empty()
        || !manifest
            .id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(
            "Template id must contain only lowercase letters, numbers, hyphens, or underscores."
                .into(),
        );
    }
    if manifest.name.trim().is_empty() {
        return Err("Template name must not be empty.".into());
    }
    if manifest.category.trim().is_empty() {
        return Err("Template category must not be empty.".into());
    }
    if !["story-development", "worldbuilding", "campaign", "adventure", "encounter", "session", "gm", "manuscript", "outline", "review", "publishing", "project"]
        .contains(&manifest.scope.as_str())
    {
        return Err(format!("Template scope '{}' is not supported.", manifest.scope));
    }
    if manifest.document_type.trim().is_empty() {
        return Err("Template document_type must not be empty.".into());
    }
    if manifest.version == 0 {
        return Err("Template version must be at least 1.".into());
    }

    let allowed_types = ["text", "multiline", "list", "select", "image", "number", "boolean"];
    let mut keys = BTreeSet::new();
    for field in &manifest.fields {
        if field.key.trim().is_empty() || field.label.trim().is_empty() || field.field_type.trim().is_empty() {
            return Err("Template fields require non-empty key, label, and type values.".into());
        }
        if !allowed_types.contains(&field.field_type.as_str()) {
            return Err(format!(
                "Template field '{}' uses unsupported type '{}'.",
                field.key, field.field_type
            ));
        }
        if !keys.insert(field.key.clone()) {
            return Err(format!("Template field '{}' is duplicated.", field.key));
        }
    }

    for key in &manifest.overview_fields {
        if !keys.contains(key) {
            return Err(format!(
                "Overview field '{}' is not declared in template fields.",
                key
            ));
        }
    }

    if !manifest.image_field.trim().is_empty()
        && manifest.image_field != "image"
        && !keys.contains(&manifest.image_field)
    {
        return Err(format!(
            "Image field '{}' is not declared in template fields.",
            manifest.image_field
        ));
    }

    Ok(())
}


fn validate_user_relative_path(relative_path: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative_path);
    if relative_path.trim().is_empty() || path.is_absolute() {
        return Err("User template path must be a non-empty relative path.".into());
    }
    if path.extension().and_then(|value| value.to_str()) != Some("md") {
        return Err("User template path must end in .md.".into());
    }
    for component in path.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err("User template path must not contain traversal or absolute components.".into());
        }
    }
    Ok(path.to_path_buf())
}

fn serialize_user_template(manifest: &EntityTemplateManifest, body: &str) -> Result<String, String> {
    let mut root = BTreeMap::new();
    root.insert(
        "lithomark_template".to_string(),
        serde_yaml::to_value(manifest)
            .map_err(|e| format!("Could not serialize template metadata: {e}"))?,
    );
    let yaml = serde_yaml::to_string(&root)
        .map_err(|e| format!("Could not serialize template YAML: {e}"))?;
    Ok(format!("---\n{}---\n\n{}\n", yaml, normalize_body(body)))
}

pub fn save_user_template(
    user_templates_dir: &Path,
    input: SaveUserEntityTemplateInput,
) -> Result<EntityTemplateRegistry, String> {
    if !user_templates_dir.is_absolute() {
        return Err("Templates folder must be an absolute local path.".into());
    }
    validate_manifest(&input.manifest)?;
    let relative_path = validate_user_relative_path(&input.relative_path)?;
    let previous_relative_path = input
        .previous_relative_path
        .as_deref()
        .map(validate_user_relative_path)
        .transpose()?;

    fs::create_dir_all(user_templates_dir)
        .map_err(|e| format!("Could not create Templates folder '{}': {e}", user_templates_dir.display()))?;

    if built_in_templates().into_iter().chain(legacy_lithomark_templates())
        .any(|template| template.manifest.id == input.manifest.id)
    {
        return Err(format!(
            "Template id '{}' belongs to a built-in template and cannot be replaced.",
            input.manifest.id
        ));
    }

    let current = load_registry(user_templates_dir)?;
    if current.templates.iter().any(|template| {
        template.source == "user"
            && template.manifest.id == input.manifest.id
            && template.relative_path.as_deref() != input.previous_relative_path.as_deref()
    }) {
        return Err(format!("Template id '{}' is already in use.", input.manifest.id));
    }

    let absolute_path = user_templates_dir.join(&relative_path);
    if absolute_path.exists()
        && previous_relative_path.as_ref() != Some(&relative_path)
    {
        return Err(format!(
            "Template file '{}' already exists.",
            relative_path.to_string_lossy()
        ));
    }

    if let Some(parent) = absolute_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create template folder '{}': {e}", parent.display()))?;
    }

    let serialized = serialize_user_template(&input.manifest, &input.body)?;
    let staged = absolute_path.with_extension("md.tmp");
    fs::write(&staged, serialized)
        .map_err(|e| format!("Could not stage template '{}': {e}", absolute_path.display()))?;
    if absolute_path.exists() {
        fs::remove_file(&absolute_path)
            .map_err(|e| format!("Could not replace template '{}': {e}", absolute_path.display()))?;
    }
    fs::rename(&staged, &absolute_path)
        .map_err(|e| format!("Could not save template '{}': {e}", absolute_path.display()))?;

    if let Some(previous) = previous_relative_path {
        if previous != relative_path {
            let old_path = user_templates_dir.join(previous);
            if old_path.exists() {
                fs::remove_file(&old_path)
                    .map_err(|e| format!("Could not remove previous template '{}': {e}", old_path.display()))?;
            }
        }
    }

    load_registry(user_templates_dir)
}

pub fn delete_user_template(
    user_templates_dir: &Path,
    relative_path: &str,
) -> Result<EntityTemplateRegistry, String> {
    if !user_templates_dir.is_absolute() {
        return Err("Templates folder must be an absolute local path.".into());
    }
    let relative_path = validate_user_relative_path(relative_path)?;
    let absolute_path = user_templates_dir.join(relative_path);
    if !absolute_path.exists() {
        return Err("User template file does not exist.".into());
    }
    fs::remove_file(&absolute_path)
        .map_err(|e| format!("Could not delete template '{}': {e}", absolute_path.display()))?;
    load_registry(user_templates_dir)
}

fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let normalized = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    if !normalized.starts_with("---") {
        return Err("Template must begin with YAML frontmatter ('---').".into());
    }

    let after_first = normalized
        .strip_prefix("---\r\n")
        .or_else(|| normalized.strip_prefix("---\n"))
        .ok_or_else(|| "Template opening frontmatter delimiter must be on its own line.".to_string())?;

    let delimiter = if let Some(index) = after_first.find("\n---\n") {
        (index, 5usize)
    } else if let Some(index) = after_first.find("\n---\r\n") {
        (index, 6usize)
    } else {
        return Err("Template YAML frontmatter is not closed with '---'.".into());
    };

    let yaml = &after_first[..delimiter.0];
    let body = &after_first[delimiter.0 + delimiter.1..];
    Ok((yaml, body))
}

fn normalize_body(body: &str) -> String {
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    normalized.trim_matches('\n').to_string()
}

pub(crate) fn built_in_template(id: &str) -> Option<EntityTemplate> {
    built_in_templates()
        .into_iter()
        .chain(legacy_lithomark_templates())
        .find(|template| template.manifest.id == id)
}

fn built_in_templates() -> Vec<EntityTemplate> {
    let mut templates = vec![
        built_in(
            "character-profile",
            "Character Profile",
            "character",
            "character",
            None,
            "Deep character-development profile with a compact Overview card.",
            &["subtitle", "role", "pov", "status", "age", "occupation", "affiliation", "current_location", "narrative_role", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("role", "Role", "text"),
                ("pov", "Roleplay Perspective", "text"),
                ("status", "Status", "text"),
                ("aliases", "Aliases", "list"),
                ("age", "Age", "text"),
                ("birth_year", "Birth Year", "text"),
                ("occupation", "Occupation", "text"),
                ("affiliation", "Affiliation", "text"),
                ("current_location", "Current Location", "text"),
                ("narrative_role", "Campaign Role", "multiline"),
                ("first_appearance", "First Appearance", "text"),
                ("last_appearance", "Last Appearance", "text"),
                ("tags", "Tags", "list"),
            ],
            "# Character Summary\n\n## Canonical Identity\n\n| Field | Canon |\n| --- | --- |\n| Full name |  |\n| Birth year |  |\n| Age at campaign start |  |\n| Occupation |  |\n| Campaign role |  |\n\n## Core Character Concept\n\n## Family & Background\n\n## Education & Career\n\n## Important Life Events\n\n## Key Relationships\n\n## Professional & Practical Strengths\n\n## Personality\n\n### Strengths\n\n### Flaws\n\n### Contradictions\n\n## Psychological & Behavioral Habits\n\n## Life Beyond Adventuring\n\n## Character Development & Campaign History\n\n### Stage One\n\n### Stage Two\n\n### Stage Three\n\n### Stage Four\n\n### Stage Five\n\n## Voice & Dialogue Notes\n\n## Continuity Notes",
        ),
        built_in(
            "character-relationship",
            "Character Relationship",
            "character",
            "character",
            Some("relationship"),
            "Focused relationship dossier for the bond, history, tension, and evolution between characters.",
            &["relationship_type", "character_a", "character_b", "status", "story_role", "tags"],
            &[
                ("image", "Image", "image"),
                ("relationship_type", "Relationship Type", "text"),
                ("character_a", "Character A", "text"),
                ("character_b", "Character B", "text"),
                ("status", "Status", "text"),
                ("origin", "Origin", "text"),
                ("public_dynamic", "Public Dynamic", "multiline"),
                ("private_dynamic", "Private Dynamic", "multiline"),
                ("story_role", "Campaign Role", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Relationship Summary\n\n## Canonical Relationship\n\n| Field | Canon |\n| --- | --- |\n| Character A |  |\n| Character B |  |\n| Relationship type |  |\n| Status |  |\n\n## Origin & Shared History\n\n## Public Dynamic\n\n## Private Dynamic\n\n## Trust, Loyalty & Intimacy\n\n## Sources of Conflict\n\n## Power Balance\n\n## Secrets & Misunderstandings\n\n## Important Shared Events\n\n## Relationship Progression\n\n### Beginning State\n\n### Pressure & Change\n\n### Breaking / Turning Point\n\n### End State\n\n## Campaign Use\n\n## Dialogue & Interaction Notes\n\n## Continuity Notes",
        ),
        built_in(
            "character-arc",
            "Character Arc",
            "character",
            "character",
            Some("arc"),
            "Character transformation tracker covering internal need, external goal, pressure, turning points, and end state.",
            &["character", "arc_type", "starting_state", "ending_state", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("character", "Character", "text"),
                ("arc_type", "Arc Type", "text"),
                ("status", "Status", "text"),
                ("external_goal", "External Goal", "text"),
                ("internal_need", "Internal Need", "text"),
                ("misbelief", "Misbelief / Wound", "multiline"),
                ("starting_state", "Starting State", "multiline"),
                ("ending_state", "Ending State", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Character Arc Summary\n\n## Character & Arc Type\n\n## Starting State\n\n## External Goal\n\n## Internal Need\n\n## Wound, Fear & Misbelief\n\n## Stakes & Consequences\n\n## Arc Progression\n\n### Starting State\n\n### Challenges and Changes\n\n### Crucial Choices\n\n### Consequences\n\n### Current State\n\n## Key Turning Points\n\n## Relationships That Shape the Arc\n\n## Ending State\n\n## Thematic Function\n\n## Continuity Notes",
        ),
        built_in(
            "profession-occupation",
            "Profession & Occupation",
            "character",
            "character",
            Some("profession"),
            "Profession reference for duties, expertise, workplace culture, career path, and story-relevant practical details.",
            &["profession", "industry", "organization", "rank", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("profession", "Profession", "text"),
                ("industry", "Industry / Field", "text"),
                ("organization", "Organization / Employer", "text"),
                ("rank", "Rank / Position", "text"),
                ("status", "Status", "text"),
                ("training", "Training / Qualifications", "multiline"),
                ("skills", "Core Skills", "list"),
                ("tools", "Tools / Equipment", "list"),
                ("tags", "Tags", "list"),
            ],
            "# Profession Summary\n\n## Canonical Identity\n\n## Role & Responsibilities\n\n## Training & Qualifications\n\n## Skills & Expertise\n\n## Tools, Equipment & Resources\n\n## Workplace & Environment\n\n## Hierarchy & Authority\n\n## Colleagues, Clients & Rivals\n\n## Daily Routine\n\n## Risks & Pressures\n\n## Culture, Jargon & Practices\n\n## Career History & Advancement\n\n## Story-Relevant Capabilities\n\n## Accuracy / Research Notes\n\n## Continuity Notes",
        ),
        built_in(
            "species-ancestry",
            "Species & Ancestry",
            "character",
            "character",
            Some("species"),
            "Species, ancestry, lineage, or people-group reference combining biological, cultural, and story-facing canon.",
            &["species_name", "classification", "homeland", "lifespan", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("species_name", "Species / Ancestry Name", "text"),
                ("classification", "Classification", "text"),
                ("homeland", "Homeland / Origin", "text"),
                ("lifespan", "Typical Lifespan", "text"),
                ("status", "Status", "text"),
                ("traits", "Defining Traits", "list"),
                ("languages", "Languages", "list"),
                ("related_groups", "Related Groups", "list"),
                ("tags", "Tags", "list"),
            ],
            "# Species & Ancestry Summary\n\n## Canonical Identity\n\n## Origins & Evolution / Creation\n\n## Physical Characteristics\n\n## Life Cycle & Lifespan\n\n## Senses, Abilities & Limitations\n\n## Needs, Health & Vulnerabilities\n\n## Population & Distribution\n\n## Homeland & Environment\n\n## Culture & Social Organization\n\n## Language & Communication\n\n## Beliefs, Customs & Traditions\n\n## Relationships With Other Peoples\n\n## Lineages & Variations\n\n## Story-Relevant Canon\n\n## Continuity Notes",
        ),
        built_in(
            "location-profile",
            "Location Profile",
            "location",
            "location",
            None,
            "Deep setting/location profile with visual reference support.",
            &["subtitle", "location_type", "region", "story_role", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("location_type", "Location Type", "text"),
                ("aliases", "Aliases", "list"),
                ("region", "Region", "text"),
                ("story_role", "Campaign Role", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Location Summary\n\n## Canonical Identity\n\n| Field | Canon |\n| --- | --- |\n| Name |  |\n| Type |  |\n| Region |  |\n| Story role |  |\n\n## Physical Description\n\n## Geography & Layout\n\n## Atmosphere & Sensory Details\n\n## History\n\n## Culture & Population\n\n## Important Areas\n\n## Connected Characters\n\n## Connected Organizations\n\n## Campaign Use\n\n## Events & Scenes\n\n## Continuity Notes",
        ),
        built_in(
            "settlement-city",
            "Settlement & City",
            "location",
            "location",
            Some("settlement"),
            "Deep settlement profile for cities, towns, villages, colonies, and inhabited communities.",
            &["subtitle", "settlement_type", "region", "population", "government", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("settlement_type", "Settlement Type", "text"),
                ("region", "Region / Territory", "text"),
                ("population", "Population", "text"),
                ("government", "Government / Authority", "text"),
                ("founded", "Founded / Established", "text"),
                ("status", "Status", "text"),
                ("demonym", "Demonym", "text"),
                ("tags", "Tags", "list"),
            ],
            "# Settlement Summary\n\n## Canonical Identity\n\n| Field | Canon |\n| --- | --- |\n| Name |  |\n| Type |  |\n| Region |  |\n| Population |  |\n| Government |  |\n\n## Geography & Setting\n\n## Districts & Layout\n\n## Population & Demographics\n\n## Government & Law\n\n## Economy & Trade\n\n## Infrastructure & Services\n\n## Culture & Daily Life\n\n## Religion & Belief\n\n## History & Founding\n\n## Important Locations\n\n## Organizations & Factions\n\n## Notable Residents\n\n## Conflicts & Pressures\n\n## Campaign Use\n\n## Events & Scenes\n\n## Continuity Notes",
        ),
        built_in(
            "building-structure",
            "Building & Structure",
            "location",
            "location",
            Some("structure"),
            "Building or constructed-site profile covering layout, use, occupants, history, access, and story function.",
            &["subtitle", "structure_type", "location", "owner", "occupants", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("structure_type", "Structure Type", "text"),
                ("location", "Location", "text"),
                ("owner", "Owner / Controller", "text"),
                ("occupants", "Occupants / Users", "list"),
                ("built", "Built / Established", "text"),
                ("status", "Status", "text"),
                ("architectural_style", "Architectural Style", "text"),
                ("tags", "Tags", "list"),
            ],
            "# Building & Structure Summary\n\n## Canonical Identity\n\n## Exterior & Approach\n\n## Interior Layout\n\n## Rooms & Important Areas\n\n## Construction & Architecture\n\n## Access, Security & Defenses\n\n## Utilities & Infrastructure\n\n## Ownership & Occupants\n\n## Daily Use & Activity\n\n## History\n\n## Secrets & Hidden Areas\n\n## Connected Characters & Organizations\n\n## Campaign Use\n\n## Events & Scenes\n\n## Continuity Notes",
        ),
        built_in(
            "geography-landmark",
            "Geography & Landmark",
            "location",
            "location",
            Some("landmark"),
            "Natural feature or landmark profile for terrain, environment, access, hazards, resources, and narrative significance.",
            &["subtitle", "feature_type", "region", "climate", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("feature_type", "Feature / Landmark Type", "text"),
                ("region", "Region", "text"),
                ("climate", "Climate", "text"),
                ("elevation_depth", "Elevation / Depth", "text"),
                ("size", "Size / Extent", "text"),
                ("status", "Status", "text"),
                ("resources", "Resources", "list"),
                ("hazards", "Hazards", "list"),
                ("tags", "Tags", "list"),
            ],
            "# Geography & Landmark Summary\n\n## Canonical Identity\n\n## Physical Description\n\n## Location & Boundaries\n\n## Terrain & Geology\n\n## Climate & Weather\n\n## Flora & Fauna\n\n## Resources\n\n## Hazards & Obstacles\n\n## Access & Travel\n\n## Human / Cultural Presence\n\n## History & Legends\n\n## Connected Locations\n\n## Campaign Use\n\n## Events & Scenes\n\n## Continuity Notes",
        ),
        built_in(
            "region-territory",
            "Region & Territory",
            "location",
            "location",
            Some("region"),
            "Large-scale geographic or political territory profile covering boundaries, peoples, resources, governance, and conflicts.",
            &["subtitle", "region_type", "parent_region", "population", "authority", "status", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("region_type", "Region / Territory Type", "text"),
                ("parent_region", "Parent Region", "text"),
                ("population", "Population", "text"),
                ("authority", "Governing Authority", "text"),
                ("capital", "Capital / Primary Settlement", "text"),
                ("status", "Status", "text"),
                ("languages", "Languages", "list"),
                ("resources", "Resources", "list"),
                ("tags", "Tags", "list"),
            ],
            "# Region & Territory Summary\n\n## Canonical Identity\n\n## Boundaries & Geography\n\n## Climate & Environment\n\n## Population & Peoples\n\n## Settlements & Important Locations\n\n## Government & Administration\n\n## Economy & Resources\n\n## Infrastructure & Travel\n\n## Culture & Identity\n\n## History\n\n## Neighboring Regions & Relations\n\n## Conflicts & Disputes\n\n## Strategic Importance\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "organization-profile",
            "Organization Profile",
            "organization",
            "organization",
            None,
            "Organization, institution, faction, company, or group profile.",
            &["subtitle", "organization_type", "leader", "headquarters", "story_role", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("organization_type", "Organization Type", "text"),
                ("aliases", "Aliases", "list"),
                ("leader", "Leader", "text"),
                ("headquarters", "Headquarters", "text"),
                ("story_role", "Campaign Role", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Organization Summary\n\n## Canonical Identity\n\n## Purpose & Mission\n\n## History\n\n## Structure\n\n## Leadership\n\n## Membership\n\n## Resources & Influence\n\n## Culture & Internal Politics\n\n## Allies & Rivals\n\n## Campaign Use\n\n## Key Events\n\n## Continuity Notes",
        ),
        built_in(
            "government-profile",
            "Government Profile",
            "organization",
            "organization",
            Some("government"),
            "Government-specific organization profile.",
            &["subtitle", "government_type", "capital", "leader", "story_role", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("organization_type", "Organization Type", "text"),
                ("government_type", "Government Type", "text"),
                ("capital", "Capital", "text"),
                ("leader", "Leader", "text"),
                ("institutions", "Institutions", "list"),
                ("story_role", "Campaign Role", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Government Summary\n\n## Canonical Identity\n\n## Territory & Population\n\n## History\n\n## Constitutional Structure\n\n## Leadership\n\n## Institutions\n\n## Laws & Justice\n\n## Economy & Resources\n\n## Military & Security\n\n## Political Culture\n\n## Factions & Opposition\n\n## Foreign Relations\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "military-security-organization",
            "Military & Security Organization",
            "organization",
            "organization",
            Some("military-security"),
            "Military, intelligence, law-enforcement, guard, or security organization covering command, doctrine, capabilities, culture, and operations.",
            &["subtitle", "organization_type", "leader", "headquarters", "jurisdiction", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("organization_type","Organization Type","text"),("leader","Commander / Leader","text"),("headquarters","Headquarters","text"),("jurisdiction","Jurisdiction / Theater","text"),("status","Status","text"),("branches","Branches / Units","list"),("allies","Allies","list"),("rivals","Rivals / Adversaries","list"),("tags","Tags","list")],
            "# Military & Security Organization Summary\n\n## Canonical Identity\n\n## Mission & Mandate\n\n## History & Founding\n\n## Command Structure\n\n## Branches, Units & Specialties\n\n## Ranks & Roles\n\n## Recruitment & Training\n\n## Doctrine & Standard Procedures\n\n## Equipment & Resources\n\n## Bases, Facilities & Territory\n\n## Intelligence & Communications\n\n## Culture, Traditions & Reputation\n\n## Laws, Authority & Oversight\n\n## Allies, Rivals & Adversaries\n\n## Important Personnel\n\n## Operations & Key Events\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "faction-secret-society",
            "Faction & Secret Society",
            "organization",
            "organization",
            Some("faction"),
            "Faction, movement, conspiracy, clandestine order, or secret society covering agenda, membership, methods, secrecy, and internal politics.",
            &["subtitle", "organization_type", "leader", "headquarters", "agenda", "secrecy", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("organization_type","Faction Type","text"),("leader","Leader","text"),("headquarters","Base / Headquarters","text"),("agenda","Agenda / Goal","multiline"),("secrecy","Secrecy Level","text"),("status","Status","text"),("symbols","Symbols / Identifiers","list"),("allies","Allies","list"),("rivals","Rivals","list"),("tags","Tags","list")],
            "# Faction & Secret Society Summary\n\n## Canonical Identity\n\n## Public Face\n\n## True Purpose & Agenda\n\n## Origins & History\n\n## Leadership & Power Structure\n\n## Membership & Recruitment\n\n## Ranks, Cells & Internal Divisions\n\n## Beliefs, Rules & Initiation\n\n## Methods & Operations\n\n## Resources & Influence\n\n## Symbols, Codes & Recognition\n\n## Secrecy & Counterintelligence\n\n## Allies, Rivals & Enemies\n\n## Internal Conflicts & Secrets\n\n## Important Members\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "worldbuilding-profile",
            "Worldbuilding Profile",
            "worldbuilding",
            "note",
            Some("worldbuilding"),
            "General worldbuilding reference page for setting systems and canon.",
            &["subtitle", "setting", "time_period", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("setting", "Setting", "text"),
                ("time_period", "Time Period", "text"),
                ("systems", "Systems / Rules", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Worldbuilding Summary\n\n## Canonical Principles\n\n## Setting\n\n## History\n\n## Geography\n\n## Cultures & Societies\n\n## Governments & Institutions\n\n## Economy & Technology\n\n## Belief Systems\n\n## Rules & Systems\n\n## Conflict & Pressure Points\n\n## Story-Relevant Canon\n\n## Continuity Notes",
        ),
        built_in(
            "culture-society",
            "Culture & Society",
            "worldbuilding",
            "note",
            Some("culture"),
            "Deep cultural reference covering identity, values, social structures, customs, arts, tensions, and change.",
            &["subtitle", "culture_name", "region", "population", "languages", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("culture_name","Culture / Society Name","text"),("region","Primary Region","text"),("population","Population / Scale","text"),("languages","Languages","list"),("status","Status","text"),("values","Core Values","list"),("tags","Tags","list")],
            "# Culture & Society Summary\n\n## Canonical Identity\n\n## Origins & History\n\n## Core Values & Worldview\n\n## Social Structure & Class\n\n## Family & Kinship\n\n## Daily Life\n\n## Customs & Etiquette\n\n## Clothing & Appearance\n\n## Food & Hospitality\n\n## Arts, Music & Storytelling\n\n## Education & Knowledge\n\n## Work, Trade & Economy\n\n## Law, Authority & Social Control\n\n## Beliefs & Religion\n\n## Celebrations, Traditions & Rituals\n\n## Language & Communication\n\n## Relationships With Other Cultures\n\n## Internal Tensions & Change\n\n## Story-Relevant Canon\n\n## Continuity Notes",
        ),
        built_in(
            "history-historical-event",
            "History & Historical Event",
            "worldbuilding",
            "event",
            Some("history"),
            "Historical period or event dossier tracking causes, participants, chronology, consequences, memory, and disputed accounts.",
            &["subtitle", "date", "location", "era", "participants", "importance", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("date","Date / Period","text"),("era","Era","text"),("location","Location","text"),("participants","Participants","list"),("importance","Historical Importance","multiline"),("sources","Sources / Records","list"),("tags","Tags","list")],
            "# Historical Summary\n\n## Canonical Date & Place\n\n## Historical Context\n\n## Causes & Preconditions\n\n## Major Participants\n\n## Chronology\n\n## What Happened\n\n## Immediate Consequences\n\n## Long-Term Consequences\n\n## Political & Social Impact\n\n## Cultural Memory & Interpretation\n\n## Disputed Accounts & Uncertainties\n\n## Surviving Evidence & Sources\n\n## Connections to Present-Day Story\n\n## Continuity Notes",
        ),
        built_in(
            "timeline-event",
            "Timeline Event",
            "worldbuilding",
            "event",
            Some("timeline"),
            "Precise chronology entry for canon events, sequence dependencies, participants, and continuity.",
            &["subtitle", "date", "time", "location", "participants", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("date","Date","text"),("time","Time","text"),("location","Location","text"),("participants","Participants","list"),("status","Canon Status","text"),("preceded_by","Preceded By","text"),("followed_by","Followed By","text"),("tags","Tags","list")],
            "# Event Summary\n\n## Canonical Date, Time & Place\n\n## Participants\n\n## Preconditions\n\n## Event Sequence\n\n## Immediate Outcome\n\n## Consequences\n\n## Preceded By\n\n## Followed By\n\n## Related Adventure Threads\n\n## Evidence / Source of Canon\n\n## Continuity Checks",
        ),
        built_in(
            "magic-system",
            "Magic System",
            "worldbuilding",
            "note",
            Some("magic"),
            "Rules-first supernatural system covering source, access, costs, limits, techniques, institutions, and narrative constraints.",
            &["subtitle", "system_name", "source", "availability", "cost", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("system_name","System Name","text"),("source","Source of Power","text"),("availability","Who Can Use It","text"),("cost","Cost / Price","text"),("status","Status","text"),("disciplines","Disciplines / Schools","list"),("limitations","Key Limitations","list"),("tags","Tags","list")],
            "# Magic System Summary\n\n## Canonical Principles\n\n## Source of Power\n\n## Access & Eligibility\n\n## Rules & Mechanics\n\n## Costs & Consequences\n\n## Limits & Failure Conditions\n\n## Disciplines, Schools & Techniques\n\n## Learning & Mastery\n\n## Tools, Materials & Foci\n\n## Institutions & Practitioners\n\n## Social & Political Effects\n\n## Interaction With Technology / Nature\n\n## Known Exceptions & Edge Cases\n\n## Story-Relevant Applications\n\n## Continuity Rules",
        ),
        built_in(
            "technology-science",
            "Technology & Science",
            "worldbuilding",
            "note",
            Some("technology"),
            "Scientific or technological system reference covering principles, capabilities, limits, access, infrastructure, and consequences.",
            &["subtitle", "technology_name", "field", "inventor", "availability", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("technology_name","Technology / Science Name","text"),("field","Field / Discipline","text"),("inventor","Inventor / Originator","text"),("availability","Availability","text"),("status","Development Status","text"),("capabilities","Capabilities","list"),("limitations","Limitations","list"),("tags","Tags","list")],
            "# Technology & Science Summary\n\n## Canonical Principles\n\n## Scientific Basis / Operating Theory\n\n## Origin & Development\n\n## Components & Requirements\n\n## Capabilities\n\n## Limitations & Failure Modes\n\n## Power, Fuel & Resources\n\n## Manufacture & Maintenance\n\n## Availability & Cost\n\n## Users & Institutions\n\n## Infrastructure\n\n## Social, Economic & Political Effects\n\n## Safety, Ethics & Regulation\n\n## Competing / Related Technologies\n\n## Story-Relevant Applications\n\n## Continuity Rules",
        ),
        built_in(
            "religion-mythology",
            "Religion & Mythology",
            "worldbuilding",
            "note",
            Some("religion"),
            "Belief-system reference covering cosmology, doctrine, deities, institutions, practices, texts, schisms, and cultural influence.",
            &["subtitle", "faith_name", "type", "origin", "followers", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("faith_name","Religion / Mythology Name","text"),("type","Type / Tradition","text"),("origin","Origin","text"),("followers","Followers / Reach","text"),("status","Status","text"),("deities","Deities / Sacred Figures","list"),("texts","Sacred Texts","list"),("tags","Tags","list")],
            "# Religion & Mythology Summary\n\n## Canonical Identity\n\n## Origins & Founding\n\n## Cosmology & Creation\n\n## Deities, Spirits & Sacred Figures\n\n## Core Beliefs & Doctrine\n\n## Morality, Sin & Virtue\n\n## Afterlife & Death\n\n## Sacred Texts & Oral Traditions\n\n## Worship & Ritual Practice\n\n## Holy Days & Festivals\n\n## Clergy & Institutions\n\n## Sacred Places & Objects\n\n## Sects, Denominations & Schisms\n\n## Heresy, Doubt & Reform\n\n## Relationship With Government & Society\n\n## Myths & Legends\n\n## Story-Relevant Canon\n\n## Continuity Notes",
        ),
        built_in(
            "language-profile",
            "Language",
            "worldbuilding",
            "note",
            Some("language"),
            "Language reference covering speakers, structure, writing, naming, registers, history, and usable story vocabulary.",
            &["subtitle", "language_name", "family", "speakers", "regions", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("language_name","Language Name","text"),("family","Language Family","text"),("speakers","Speakers","text"),("regions","Regions","list"),("status","Status","text"),("scripts","Writing Systems","list"),("related_languages","Related Languages","list"),("tags","Tags","list")],
            "# Language Summary\n\n## Canonical Identity\n\n## Speakers & Distribution\n\n## History & Development\n\n## Sounds & Pronunciation\n\n## Grammar & Sentence Structure\n\n## Vocabulary & Word Formation\n\n## Writing System & Orthography\n\n## Registers, Dialects & Slang\n\n## Naming Conventions\n\n## Idioms, Proverbs & Expressions\n\n## Titles & Forms of Address\n\n## Related Languages\n\n## Example Vocabulary\n\n## Example Phrases\n\n## Story Usage & Consistency Rules",
        ),
        built_in(
            "disease-condition",
            "Disease & Condition",
            "worldbuilding",
            "note",
            Some("condition"),
            "Disease, injury, syndrome, curse, or condition reference covering cause, progression, treatment, impact, and story continuity.",
            &["subtitle", "condition_name", "condition_type", "cause", "transmission", "severity", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("condition_name","Condition Name","text"),("condition_type","Condition Type","text"),("cause","Cause / Etiology","text"),("transmission","Transmission","text"),("severity","Severity","text"),("status","Status","text"),("symptoms","Key Symptoms","list"),("treatments","Treatments","list"),("tags","Tags","list")],
            "# Disease & Condition Summary\n\n## Canonical Identity\n\n## Cause / Etiology\n\n## Transmission & Exposure\n\n## Risk Factors\n\n## Signs & Symptoms\n\n## Progression & Stages\n\n## Diagnosis\n\n## Treatment & Management\n\n## Prognosis & Recovery\n\n## Prevention\n\n## Complications & Long-Term Effects\n\n## Social & Cultural Impact\n\n## History & Known Cases\n\n## Story-Relevant Constraints\n\n## Accuracy / Research Notes\n\n## Continuity Notes",
        ),
        built_in(
            "natural-environment-ecosystem",
            "Natural Environment & Ecosystem",
            "worldbuilding",
            "note",
            Some("ecosystem"),
            "Ecology reference covering climate, habitats, species, food webs, cycles, resources, hazards, and human pressures.",
            &["subtitle", "ecosystem_name", "biome", "region", "climate", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("ecosystem_name","Environment / Ecosystem Name","text"),("biome","Biome / Type","text"),("region","Region","text"),("climate","Climate","text"),("status","Ecological Status","text"),("key_species","Key Species","list"),("resources","Resources","list"),("hazards","Hazards","list"),("tags","Tags","list")],
            "# Environment & Ecosystem Summary\n\n## Canonical Identity\n\n## Geography & Extent\n\n## Climate & Seasonal Cycles\n\n## Terrain, Water & Soils\n\n## Habitats & Ecological Zones\n\n## Flora\n\n## Fauna\n\n## Food Web & Ecological Relationships\n\n## Migration & Life Cycles\n\n## Resources\n\n## Natural Hazards\n\n## Human / Sentient Interaction\n\n## Environmental Pressures & Change\n\n## Conservation / Exploitation\n\n## Story-Relevant Canon\n\n## Continuity Notes",
        ),
        built_in(
            "object-item-profile",
            "Object / Item Profile",
            "object",
            "object",
            Some("item"),
            "Significant object, artifact, evidence item, tool, or possession.",
            &["subtitle", "object_type", "owner", "origin", "story_role", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("object_type", "Object Type", "text"),
                ("owner", "Owner", "text"),
                ("origin", "Origin", "text"),
                ("story_role", "Campaign Role", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Object Summary\n\n## Canonical Identity\n\n## Physical Description\n\n## Origin & History\n\n## Ownership & Custody\n\n## Capabilities / Properties\n\n## Symbolic Meaning\n\n## Campaign Use\n\n## Appearances & Events\n\n## Continuity Notes",
        ),
        built_in(
            "artifact-relic",
            "Artifact & Relic",
            "object",
            "object",
            Some("artifact"),
            "Unique or historically significant artifact covering provenance, properties, ownership, symbolism, secrets, and story consequences.",
            &["subtitle", "object_type", "creator", "origin", "owner", "current_location", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("object_type","Artifact Type","text"),("creator","Creator / Maker","text"),("origin","Origin","text"),("owner","Owner / Custodian","text"),("current_location","Current Location","text"),("status","Status","text"),("materials","Materials","list"),("properties","Properties / Powers","list"),("tags","Tags","list")],
            "# Artifact & Relic Summary\n\n## Canonical Identity\n\n## Physical Description\n\n## Materials & Construction\n\n## Creation & Origin\n\n## Provenance & Ownership History\n\n## Properties, Powers & Capabilities\n\n## Limits, Costs & Risks\n\n## Activation / Use\n\n## Symbolism & Cultural Meaning\n\n## Legends & Conflicting Accounts\n\n## Current Owner & Location\n\n## Secrets & Hidden Properties\n\n## Campaign Use\n\n## Appearances & Key Events\n\n## Continuity Notes",
        ),
        built_in(
            "vehicle-vessel",
            "Vehicle & Vessel",
            "object",
            "object",
            Some("vehicle"),
            "Vehicle, ship, aircraft, spacecraft, train, or other conveyance covering design, capabilities, crew, operations, and history.",
            &["subtitle", "vehicle_type", "manufacturer", "owner", "operator", "home_base", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("vehicle_type","Vehicle / Vessel Type","text"),("manufacturer","Manufacturer / Builder","text"),("owner","Owner","text"),("operator","Operator / Captain","text"),("home_base","Home Base / Port","text"),("status","Status","text"),("capacity","Capacity","text"),("crew","Crew","list"),("tags","Tags","list")],
            "# Vehicle & Vessel Summary\n\n## Canonical Identity\n\n## Appearance & Design\n\n## Construction & History\n\n## Propulsion / Movement\n\n## Performance & Range\n\n## Interior Layout & Compartments\n\n## Systems & Equipment\n\n## Defenses / Armament\n\n## Crew, Passengers & Capacity\n\n## Ownership & Operation\n\n## Maintenance, Fuel & Logistics\n\n## Limitations & Vulnerabilities\n\n## Home Base, Routes & Operating Area\n\n## Campaign Use\n\n## Key Journeys & Events\n\n## Continuity Notes",
        ),
        built_in(
            "material-resource",
            "Material & Resource",
            "object",
            "object",
            Some("material"),
            "Material, commodity, mineral, fuel, biological resource, or strategic substance covering properties, sources, processing, value, and impact.",
            &["subtitle", "material_type", "source", "rarity", "value", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("material_type","Material / Resource Type","text"),("source","Primary Source","text"),("rarity","Rarity / Availability","text"),("value","Value / Importance","text"),("status","Status","text"),("properties","Properties","list"),("uses","Uses","list"),("hazards","Hazards","list"),("tags","Tags","list")],
            "# Material & Resource Summary\n\n## Canonical Identity\n\n## Physical / Chemical Properties\n\n## Natural or Manufactured Origin\n\n## Sources & Distribution\n\n## Extraction / Production\n\n## Refining & Processing\n\n## Uses & Applications\n\n## Rarity, Supply & Value\n\n## Trade & Economic Importance\n\n## Strategic / Political Importance\n\n## Hazards & Handling\n\n## Environmental & Social Impact\n\n## Substitutes & Related Materials\n\n## Story-Relevant Canon\n\n## Continuity Notes",
        ),
        built_in(
            "event-profile",
            "Event Profile",
            "event",
            "event",
            None,
            "Historical, timeline, backstory, or story event.",
            &["subtitle", "date", "location", "importance", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("date", "Date / Time", "text"),
                ("location", "Location", "text"),
                ("participants", "Participants", "list"),
                ("importance", "Importance", "multiline"),
                ("tags", "Tags", "list"),
            ],
            "# Event Summary\n\n## Canonical Date & Place\n\n## Participants\n\n## What Happened\n\n## Causes\n\n## Immediate Consequences\n\n## Long-Term Consequences\n\n## Character Impact\n\n## Evidence / Sources\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "general-reference",
            "General Reference",
            "general",
            "note",
            Some("reference"),
            "Flexible reference article for canon that does not fit a specialized Story Development template.",
            &["subtitle", "reference_type", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("reference_type","Reference Type","text"),("status","Canon Status","text"),("aliases","Aliases / Alternate Names","list"),("related_entries","Related Entries","list"),("tags","Tags","list")],
            "# Reference Summary\n\n## Canonical Definition\n\n## Description\n\n## Background & Context\n\n## History\n\n## Important Details\n\n## Rules & Constraints\n\n## Related People, Places & Organizations\n\n## Related Events & Concepts\n\n## Story Relevance\n\n## Open Questions\n\n## Sources & Research Notes\n\n## Continuity Notes",
        ),
        built_in(
            "in-world-document",
            "In-World Document",
            "general",
            "note",
            Some("in-world-document"),
            "Diegetic document profile for letters, reports, articles, records, journals, laws, messages, files, and other texts that exist inside the story world.",
            &["subtitle", "document_type", "author", "recipient", "date", "origin", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("document_type","Document Type","text"),("author","Author / Issuer","text"),("recipient","Recipient / Audience","text"),("date","Date","text"),("origin","Place / Organization of Origin","text"),("status","Authenticity / Status","text"),("classification","Classification / Access","text"),("tags","Tags","list")],
            "# Document Summary\n\n## Canonical Identity\n\n## Physical / Digital Description\n\n## Author, Issuer & Provenance\n\n## Intended Recipient / Audience\n\n## Date & Circumstances\n\n## In-World Content\n\n## Meaning & Interpretation\n\n## Reliability, Bias & Authenticity\n\n## Chain of Custody / Discovery\n\n## Who Knows About It\n\n## Secrets, Redactions & Hidden Information\n\n## Campaign Use\n\n## Appearances & References\n\n## Continuity Notes",
        ),
        built_in(
            "plot-thread-mystery",
            "Plot Thread & Mystery",
            "general",
            "note",
            Some("plot-thread"),
            "Story-development tracker for mysteries, questions, clues, reveals, red herrings, dependencies, and resolution.",
            &["subtitle", "thread_type", "status", "introduced", "resolved", "stakes", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("thread_type","Thread / Mystery Type","text"),("status","Status","text"),("introduced","Introduced In","text"),("resolved","Resolved In","text"),("stakes","Stakes","multiline"),("characters","Key Characters","list"),("locations","Key Locations","list"),("tags","Tags","list")],
            "# Plot Thread / Mystery Summary\n\n## Central Question or Conflict\n\n## What the Players Know\n\n## What the Player Characters Know\n\n## Hidden Truth / Intended Resolution\n\n## Origin & Setup\n\n## Stakes\n\n## Clues & Evidence\n\n## Red Herrings & Misdirection\n\n## Escalation & Complications\n\n## Dependencies & Connected Threads\n\n## Character Involvement\n\n## Reveal Plan\n\n## Resolution & Consequences\n\n## Scene / Chapter Tracking\n\n## Continuity Checks",
        ),
        built_in(
            "conflict-war",
            "Conflict & War",
            "general",
            "note",
            Some("conflict"),
            "Conflict dossier for wars, feuds, rebellions, political struggles, rivalries, and other sustained contests.",
            &["subtitle", "conflict_type", "start_date", "end_date", "status", "location", "belligerents", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("conflict_type","Conflict Type","text"),("start_date","Start Date / Period","text"),("end_date","End Date / Period","text"),("status","Status","text"),("location","Primary Theater / Location","text"),("belligerents","Sides / Belligerents","list"),("leaders","Leaders","list"),("tags","Tags","list")],
            "# Conflict & War Summary\n\n## Canonical Identity\n\n## Background & Root Causes\n\n## Trigger / Inciting Incident\n\n## Sides, Factions & Leadership\n\n## Goals & Stakes\n\n## Resources & Capabilities\n\n## Geography & Theaters\n\n## Strategy, Tactics & Methods\n\n## Chronology & Major Events\n\n## Turning Points\n\n## Civilian & Social Impact\n\n## Political & Economic Impact\n\n## Alliances & External Actors\n\n## Outcome / Current State\n\n## Long-Term Consequences\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "tradition-ritual",
            "Tradition & Ritual",
            "general",
            "note",
            Some("tradition"),
            "Tradition, ceremony, custom, rite, festival, or recurring social practice covering meaning, participants, procedure, variation, and story use.",
            &["subtitle", "tradition_type", "culture", "location", "frequency", "status", "tags"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("tradition_type","Tradition / Ritual Type","text"),("culture","Culture / Community","text"),("location","Location","text"),("frequency","Frequency / Timing","text"),("status","Status","text"),("participants","Participants","list"),("symbols","Symbols / Objects","list"),("tags","Tags","list")],
            "# Tradition & Ritual Summary\n\n## Canonical Identity\n\n## Origin & History\n\n## Purpose & Meaning\n\n## Participants & Roles\n\n## Timing & Occasion\n\n## Location & Setting\n\n## Preparation\n\n## Sequence / Procedure\n\n## Objects, Clothing & Symbols\n\n## Words, Music & Performance\n\n## Food, Gifts & Offerings\n\n## Rules, Taboos & Expectations\n\n## Variations & Regional Differences\n\n## Social Status & Controversies\n\n## Campaign Use\n\n## Continuity Notes",
        ),
        built_in(
            "research-profile",
            "Research Profile",
            "research",
            "research",
            None,
            "Research topic, source notes, and application tracker.",
            &["subtitle", "topic", "status", "source", "tags"],
            &[
                ("image", "Image", "image"),
                ("subtitle", "Subtitle", "text"),
                ("topic", "Topic", "text"),
                ("source", "Source", "text"),
                ("status", "Status", "text"),
                ("tags", "Tags", "list"),
            ],
            "# Research Question\n\n## Why It Matters\n\n## Working Notes\n\n## Facts & Findings\n\n## Sources\n\n## Unresolved Questions\n\n## Application to the Campaign\n\n## Canon / Accuracy Notes",
        ),
    ];

    // Novel/project publishing and manuscript presets are legacy ID fallbacks only.
    templates.extend(catalyst_additional_templates());
    templates.extend(campaign_templates());
    templates
}

fn project_templates() -> Vec<EntityTemplate> {
    vec![
        built_in_scoped(
            "project-novel",
            "Novel",
            "project",
            "book",
            "project",
            Some("novel"),
            "Full-length single-book fiction project foundation with manuscript, story-development, research, review, publishing, and project-support areas.",
            &["genre", "status", "target_words", "series", "series_number", "tags"],
            &[("genre","Genre","text"),("subgenre","Subgenre","text"),("status","Project Status","text"),("target_words","Target Word Count","number"),("series","Series","text"),("series_number","Series Number","text"),("premise","Premise","multiline"),("synopsis","Synopsis","multiline"),("tags","Tags","list")],
            "# Project Foundation\n\n## Premise\n\n## Synopsis\n\n## Goals\n\n## Manuscript\n\n## Story Development\n\n## Research\n\n## Outline / Structure\n\n## Review & Revision\n\n## Publishing\n\n## Project Notes",
        ),
        built_in_scoped(
            "project-novella",
            "Novella",
            "project",
            "book",
            "project",
            Some("novella"),
            "Novella project foundation using the same durable writing system as a novel without assuming novel-length structure or complexity.",
            &["genre", "status", "target_words", "series", "series_number", "tags"],
            &[("genre","Genre","text"),("subgenre","Subgenre","text"),("status","Project Status","text"),("target_words","Target Word Count","number"),("series","Series","text"),("series_number","Series Number","text"),("premise","Premise","multiline"),("synopsis","Synopsis","multiline"),("tags","Tags","list")],
            "# Project Foundation\n\n## Premise\n\n## Synopsis\n\n## Goals\n\n## Manuscript\n\n## Story Development\n\n## Research\n\n## Outline / Structure\n\n## Review & Revision\n\n## Publishing\n\n## Project Notes",
        ),
        built_in_scoped(
            "project-short-story",
            "Short Story",
            "project",
            "book",
            "project",
            Some("short-story"),
            "Compact fiction project foundation for short stories while retaining optional Story Development, research, review, and publishing capabilities.",
            &["genre", "status", "target_words", "collection", "tags"],
            &[("genre","Genre","text"),("subgenre","Subgenre","text"),("status","Project Status","text"),("target_words","Target Word Count","number"),("collection","Collection / Anthology","text"),("premise","Premise","multiline"),("synopsis","Synopsis","multiline"),("submission_target","Submission Target","text"),("tags","Tags","list")],
            "# Project Foundation\n\n## Premise\n\n## Synopsis\n\n## Goals\n\n## Manuscript\n\n## Story Development\n\n## Research\n\n## Review & Revision\n\n## Publishing / Submission\n\n## Project Notes",
        ),
        built_in_scoped(
            "project-series",
            "Series",
            "project",
            "series",
            "project",
            Some("series"),
            "Series-level foundation for shared canon, chronology, recurring entities, cross-book arcs, research, and individual book references without merging book manuscript authority.",
            &["genre", "status", "planned_books", "series_arc", "tags"],
            &[("genre","Genre","text"),("subgenre","Subgenre","text"),("status","Series Status","text"),("planned_books","Planned Books","number"),("series_arc","Series Arc","multiline"),("books","Books","list"),("recurring_characters","Recurring Characters","list"),("recurring_locations","Recurring Locations","list"),("shared_threads","Shared Plot Threads","list"),("tags","Tags","list")],
            "# Series Foundation\n\n## Series Premise\n\n## Series Arc\n\n## Books\n\n## Shared Canon\n\n## Master Chronology\n\n## Recurring Characters\n\n## Recurring Locations\n\n## Organizations & Factions\n\n## Cross-Book Plot Threads\n\n## Shared Research\n\n## Publishing / Branding Notes\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "project-nonfiction-book",
            "Nonfiction Book",
            "project",
            "book",
            "project",
            Some("nonfiction"),
            "Long-form nonfiction project foundation centered on argument, audience, research, sources, chapter development, review, and publication.",
            &["category", "status", "target_words", "audience", "central_thesis", "tags"],
            &[("category","Category / Subject","text"),("status","Project Status","text"),("target_words","Target Word Count","number"),("audience","Intended Audience","text"),("central_thesis","Central Thesis / Promise","multiline"),("synopsis","Overview / Synopsis","multiline"),("source_standard","Citation / Source Standard","text"),("tags","Tags","list")],
            "# Project Foundation\n\n## Central Thesis / Reader Promise\n\n## Intended Audience\n\n## Scope\n\n## Chapter Plan\n\n## Research & Sources\n\n## Interviews / Primary Material\n\n## Claims Requiring Verification\n\n## Review & Fact Checking\n\n## Publishing\n\n## Project Notes",
        ),
        built_in_scoped(
            "project-collection",
            "Collection / Anthology",
            "project",
            "collection",
            "project",
            Some("collection"),
            "Project foundation for a collection or anthology of multiple short works with shared ordering, editorial, metadata, and publication planning.",
            &["collection_type", "status", "target_words", "theme", "tags"],
            &[("collection_type","Collection Type","text"),("status","Project Status","text"),("target_words","Target Word Count","number"),("theme","Theme / Unifying Concept","multiline"),("works","Works","list"),("introduction","Introduction / Framing","multiline"),("tags","Tags","list")],
            "# Collection Foundation\n\n## Concept / Theme\n\n## Included Works\n\n## Ordering\n\n## Introduction / Framing\n\n## Shared Research\n\n## Editorial Consistency\n\n## Review & Revision\n\n## Front / Back Matter\n\n## Publishing\n\n## Project Notes",
        ),
    ]
}

fn publishing_templates() -> Vec<EntityTemplate> {
    vec![
        built_in_scoped(
            "publishing-export-profile",
            "Export Profile",
            "publishing",
            "export",
            "export-profile",
            None,
            "Reusable export configuration describing output format, included manuscript material, metadata, and destination behavior without altering source documents.",
            &["output_format", "status", "manuscript_version", "include_front_matter", "include_back_matter", "tags"],
            &[("output_format","Output Format","text"),("status","Status","text"),("manuscript_version","Manuscript Version","text"),("include_front_matter","Include Front Matter","boolean"),("include_back_matter","Include Back Matter","boolean"),("include_scene_breaks","Include Scene Breaks","boolean"),("include_chapter_titles","Include Chapter Titles","boolean"),("destination","Export Destination","text"),("tags","Tags","list")],
            "# Export Profile\n\n## Purpose\n\n## Output Format\n\n## Manuscript Selection\n\n## Front Matter\n\n## Back Matter\n\n## Chapter & Scene Handling\n\n## Metadata\n\n## Images & Assets\n\n## Output Naming\n\n## Destination\n\n## Validation Notes",
        ),
        built_in_scoped(
            "publishing-docx-profile",
            "DOCX Export",
            "publishing",
            "export",
            "export-profile",
            Some("docx"),
            "DOCX export profile for editable manuscripts, editorial handoff, submissions, and other word-processing workflows.",
            &["status", "manuscript_version", "page_size", "font_family", "font_size", "line_spacing", "tags"],
            &[("status","Status","text"),("manuscript_version","Manuscript Version","text"),("page_size","Page Size","text"),("font_family","Body Font","text"),("font_size","Body Font Size","number"),("line_spacing","Line Spacing","text"),("first_line_indent","First-Line Indent","text"),("chapter_start","Chapter Start","text"),("include_headers","Include Headers","boolean"),("include_page_numbers","Include Page Numbers","boolean"),("tags","Tags","list")],
            "# DOCX Export\n\n## Intended Use\n\n## Page Setup\n\n## Body Typography\n\n## Paragraph Formatting\n\n## Chapter Openers\n\n## Scene Breaks\n\n## Headers & Footers\n\n## Page Numbers\n\n## Front / Back Matter\n\n## Images\n\n## Export Validation",
        ),
        built_in_scoped(
            "publishing-epub-profile",
            "EPUB / Ebook Export",
            "publishing",
            "ebook",
            "export-profile",
            Some("epub"),
            "Reflowable ebook export profile for EPUB-oriented structure, navigation, metadata, images, and reader-device compatibility.",
            &["status", "manuscript_version", "language", "publisher", "isbn", "cover_image", "tags"],
            &[("status","Status","text"),("manuscript_version","Manuscript Version","text"),("language","Language","text"),("publisher","Publisher / Imprint","text"),("isbn","ISBN","text"),("cover_image","Cover Image","image"),("include_toc","Include Navigation / TOC","boolean"),("include_front_matter","Include Front Matter","boolean"),("include_back_matter","Include Back Matter","boolean"),("tags","Tags","list")],
            "# EPUB / Ebook Export\n\n## Publication Metadata\n\n## Cover\n\n## Navigation & Table of Contents\n\n## Chapter Structure\n\n## Scene Breaks\n\n## Typography Guidance\n\n## Images & Assets\n\n## Front Matter\n\n## Back Matter\n\n## Accessibility\n\n## Device / Reader Validation",
        ),
        built_in_scoped(
            "publishing-print-profile",
            "Print Book Format",
            "publishing",
            "print",
            "format-profile",
            None,
            "Print interior formatting profile for trim, margins, typography, chapter openers, running elements, and print-preview behavior.",
            &["status", "manuscript_version", "trim_size", "binding", "body_font", "body_font_size", "tags"],
            &[("status","Status","text"),("manuscript_version","Manuscript Version","text"),("trim_size","Trim Size","text"),("binding","Binding","text"),("body_font","Body Font","text"),("body_font_size","Body Font Size","number"),("line_spacing","Line Spacing","text"),("inside_margin","Inside Margin","text"),("outside_margin","Outside Margin","text"),("top_margin","Top Margin","text"),("bottom_margin","Bottom Margin","text"),("chapter_start","Chapter Start","text"),("tags","Tags","list")],
            "# Print Book Format\n\n## Trim & Binding\n\n## Margins & Gutter\n\n## Body Typography\n\n## Paragraph Style\n\n## Chapter Openers\n\n## Scene Breaks\n\n## Running Heads\n\n## Page Numbers\n\n## Front Matter\n\n## Back Matter\n\n## Images & Decorative Elements\n\n## Print Preview / Validation",
        ),
        built_in_scoped(
            "publishing-submission-manuscript",
            "Submission Manuscript",
            "publishing",
            "submission",
            "format-profile",
            None,
            "Standardized submission-format profile for agents, editors, publishers, contests, or other manuscript recipients.",
            &["status", "recipient", "guideline_source", "page_size", "font_family", "font_size", "tags"],
            &[("status","Status","text"),("recipient","Recipient / Market","text"),("guideline_source","Guideline Source","text"),("page_size","Page Size","text"),("font_family","Font Family","text"),("font_size","Font Size","number"),("line_spacing","Line Spacing","text"),("margin","Margins","text"),("header_format","Header Format","text"),("include_word_count","Include Word Count","boolean"),("include_contact_info","Include Contact Information","boolean"),("tags","Tags","list")],
            "# Submission Manuscript\n\n## Recipient / Market\n\n## Submission Guidelines\n\n## Page Setup\n\n## Typography\n\n## Paragraph Formatting\n\n## Header / Footer\n\n## Title Page\n\n## Chapter Openers\n\n## Scene Breaks\n\n## Required Metadata\n\n## Pre-Submission Checklist",
        ),
        built_in_scoped(
            "publishing-front-matter-plan",
            "Front Matter Plan",
            "publishing",
            "book-matter",
            "publication-plan",
            Some("front-matter"),
            "Publication-order plan for title pages, copyright, dedication, epigraph, contents, foreword, preface, acknowledgments, and other front matter.",
            &["status", "edition", "format", "tags"],
            &[("status","Status","text"),("edition","Edition","text"),("format","Target Format","text"),("include_half_title","Half Title","boolean"),("include_title_page","Title Page","boolean"),("include_copyright","Copyright Page","boolean"),("include_dedication","Dedication","boolean"),("include_epigraph","Epigraph","boolean"),("include_toc","Table of Contents","boolean"),("include_foreword","Foreword","boolean"),("include_preface","Preface","boolean"),("tags","Tags","list")],
            "# Front Matter Plan\n\n## Order\n\n## Half Title\n\n## Title Page\n\n## Copyright Page\n\n## Dedication\n\n## Epigraph\n\n## Table of Contents\n\n## Foreword / Preface\n\n## Acknowledgments\n\n## Other Front Matter\n\n## Format-Specific Notes",
        ),
        built_in_scoped(
            "publishing-back-matter-plan",
            "Back Matter Plan",
            "publishing",
            "book-matter",
            "publication-plan",
            Some("back-matter"),
            "Publication-order plan for acknowledgments, appendices, notes, glossary, bibliography, author material, series material, and other back matter.",
            &["status", "edition", "format", "tags"],
            &[("status","Status","text"),("edition","Edition","text"),("format","Target Format","text"),("include_acknowledgments","Acknowledgments","boolean"),("include_appendix","Appendix","boolean"),("include_glossary","Glossary","boolean"),("include_bibliography","Bibliography / Sources","boolean"),("include_author_bio","Author Bio","boolean"),("include_also_by","Also By","boolean"),("include_series_info","Series Information","boolean"),("tags","Tags","list")],
            "# Back Matter Plan\n\n## Order\n\n## Acknowledgments\n\n## Appendices\n\n## Notes\n\n## Glossary\n\n## Bibliography / Sources\n\n## About the Author\n\n## Also By\n\n## Series / Next Book\n\n## Other Back Matter\n\n## Format-Specific Notes",
        ),
        built_in_scoped(
            "publishing-publication-metadata",
            "Publication Metadata",
            "publishing",
            "metadata",
            "publication-metadata",
            None,
            "Book-level publication metadata used by export and publishing workflows without making any external publishing platform authoritative.",
            &["title", "subtitle", "author_name", "language", "publisher", "publication_date", "isbn", "tags"],
            &[("title","Publication Title","text"),("subtitle","Subtitle","text"),("author_name","Author / Pen Name","text"),("language","Language","text"),("publisher","Publisher / Imprint","text"),("publication_date","Publication Date","text"),("isbn","ISBN","text"),("edition","Edition","text"),("series","Series","text"),("series_number","Series Number","text"),("description","Book Description","multiline"),("keywords","Keywords","list"),("categories","Categories","list"),("cover_image","Cover Image","image"),("tags","Tags","list")],
            "# Publication Metadata\n\n## Identity\n\n## Author / Contributor Credits\n\n## Edition & Series\n\n## Description\n\n## Categories & Keywords\n\n## Identifiers\n\n## Publisher / Imprint\n\n## Publication Date\n\n## Rights & Copyright\n\n## Cover & Assets\n\n## Format-Specific Notes",
        ),
    ]
}

fn review_templates() -> Vec<EntityTemplate> {
    vec![
        built_in_scoped(
            "review-developmental-review",
            "Developmental Review",
            "review",
            "editorial",
            "review",
            Some("developmental"),
            "Whole-manuscript developmental assessment covering structure, character, pacing, stakes, continuity, theme, and revision priorities.",
            &["review_stage", "status", "reviewer", "manuscript_version", "overall_assessment", "priority", "tags"],
            &[("review_stage","Review Stage","text"),("status","Status","text"),("reviewer","Reviewer / Editor","text"),("manuscript_version","Manuscript Version","text"),("overall_assessment","Overall Assessment","multiline"),("priority","Revision Priority","text"),("strengths","Key Strengths","list"),("major_issues","Major Issues","list"),("tags","Tags","list")],
            "# Developmental Review\n\n## Executive Assessment\n\n## What Is Working\n\n## Story Structure\n\n## Plot & Causality\n\n## Character & Character Arcs\n\n## Stakes, Conflict & Tension\n\n## Pacing\n\n## POV & Narrative Distance\n\n## Setting & Worldbuilding\n\n## Theme & Emotional Throughline\n\n## Continuity & Logic\n\n## Opening\n\n## Middle\n\n## Ending & Payoff\n\n## Major Revision Priorities\n\n## Secondary Revision Notes\n\n## Recommended Revision Order",
        ),
        built_in_scoped(
            "review-revision-plan",
            "Revision Plan",
            "review",
            "revision",
            "revision-plan",
            None,
            "Actionable revision plan that converts editorial findings into ordered manuscript work without replacing the manuscript itself.",
            &["status", "revision_round", "manuscript_version", "priority", "target_date", "tags"],
            &[("status","Status","text"),("revision_round","Revision Round","text"),("manuscript_version","Manuscript Version","text"),("priority","Overall Priority","text"),("target_date","Target Date","text"),("goals","Revision Goals","list"),("affected_threads","Affected Plot Threads","list"),("affected_characters","Affected Characters","list"),("tags","Tags","list")],
            "# Revision Plan\n\n## Revision Goals\n\n## Must-Fix Issues\n\n## Structural Revisions\n\n## Character Revisions\n\n## Plot & Causality Revisions\n\n## Pacing & Tension Revisions\n\n## Continuity Revisions\n\n## Worldbuilding / Setting Revisions\n\n## Scene-Level Work\n\n## Line-Level Work\n\n## Chapters / Scenes Affected\n\n## Dependencies & Order of Operations\n\n## Verification Checklist\n\n## Deferred Ideas\n\n## Completion Notes",
        ),
        built_in_scoped(
            "review-beta-reader-feedback",
            "Beta Reader Feedback",
            "review",
            "reader-feedback",
            "review",
            Some("beta"),
            "Structured beta-reader response focused on reader experience, comprehension, engagement, characters, pacing, and unresolved questions.",
            &["reader", "status", "manuscript_version", "reader_type", "overall_reaction", "tags"],
            &[("reader","Beta Reader","text"),("status","Status","text"),("manuscript_version","Manuscript Version","text"),("reader_type","Reader Type / Genre Familiarity","text"),("overall_reaction","Overall Reaction","multiline"),("favorite_elements","Favorite Elements","list"),("confusing_points","Confusing Points","list"),("tags","Tags","list")],
            "# Beta Reader Feedback\n\n## Overall Reaction\n\n## Where I Was Most Engaged\n\n## Where My Attention Dropped\n\n## Characters\n\n## Plot & Story Logic\n\n## Pacing\n\n## Confusing or Unclear Moments\n\n## Emotional Response\n\n## Predictions & Expectations\n\n## Favorite Moments\n\n## Moments That Did Not Land\n\n## Questions I Still Had\n\n## Ending Reaction\n\n## Additional Notes",
        ),
        built_in_scoped(
            "review-arc-reader-feedback",
            "ARC Reader Feedback",
            "review",
            "reader-feedback",
            "review",
            Some("arc"),
            "Advance-reader feedback for a near-publication manuscript, emphasizing reader response, remaining errors, sensitivity concerns, and release readiness.",
            &["reader", "status", "manuscript_version", "format", "release_readiness", "tags"],
            &[("reader","ARC Reader","text"),("status","Status","text"),("manuscript_version","Manuscript Version","text"),("format","Reading Format","text"),("release_readiness","Release Readiness","text"),("errors_found","Errors Found","list"),("content_concerns","Content / Sensitivity Concerns","list"),("tags","Tags","list")],
            "# ARC Reader Feedback\n\n## Overall Reading Experience\n\n## Release Readiness\n\n## Remaining Errors / Typos\n\n## Continuity or Logic Concerns\n\n## Formatting / Presentation Issues\n\n## Content or Sensitivity Concerns\n\n## Strongest Elements\n\n## Weakest Remaining Elements\n\n## Ending & Satisfaction\n\n## Audience Fit\n\n## Additional Notes",
        ),
        built_in_scoped(
            "review-copyedit-pass",
            "Copyedit Pass",
            "review",
            "editorial",
            "review",
            Some("copyedit"),
            "Copyediting pass for grammar, usage, consistency, clarity, style-sheet decisions, and recurring language issues.",
            &["status", "editor", "manuscript_version", "pass_number", "style_guide", "tags"],
            &[("status","Status","text"),("editor","Editor","text"),("manuscript_version","Manuscript Version","text"),("pass_number","Pass Number","text"),("style_guide","Style Guide","text"),("recurring_issues","Recurring Issues","list"),("style_decisions","Style Decisions","list"),("tags","Tags","list")],
            "# Copyedit Pass\n\n## Scope of Pass\n\n## Grammar & Usage\n\n## Clarity & Syntax\n\n## Consistency\n\n## Names, Terms & Capitalization\n\n## Numbers, Dates & Time\n\n## Dialogue & Punctuation\n\n## Repetition & Word Choice\n\n## Style Sheet Decisions\n\n## Recurring Issues\n\n## Queries for the Author\n\n## Items Requiring Another Pass",
        ),
        built_in_scoped(
            "review-proofread-pass",
            "Proofread Pass",
            "review",
            "editorial",
            "review",
            Some("proofread"),
            "Late-stage proofreading record for typographical, punctuation, formatting, and final consistency defects.",
            &["status", "proofreader", "manuscript_version", "pass_number", "format", "tags"],
            &[("status","Status","text"),("proofreader","Proofreader","text"),("manuscript_version","Manuscript Version","text"),("pass_number","Pass Number","text"),("format","Proof Format","text"),("typos","Typos / Mechanical Errors","list"),("formatting_issues","Formatting Issues","list"),("tags","Tags","list")],
            "# Proofread Pass\n\n## Scope of Proof\n\n## Typographical Errors\n\n## Punctuation & Mechanics\n\n## Formatting Consistency\n\n## Headers / Chapter Titles\n\n## Scene Breaks\n\n## Italics, Emphasis & Special Text\n\n## Widows, Orphans & Layout Notes\n\n## Cross-References / Contents\n\n## Remaining Queries\n\n## Final Verification",
        ),
        built_in_scoped(
            "review-reviewer-package",
            "Reviewer Package",
            "review",
            "distribution",
            "review-package",
            None,
            "Project-owned specification for a beta, ARC, critique, or editorial review package and the material intentionally shared with reviewers.",
            &["package_type", "status", "manuscript_version", "audience", "due_date", "feedback_mode", "tags"],
            &[("package_type","Package Type","text"),("status","Status","text"),("manuscript_version","Manuscript Version","text"),("audience","Audience / Reviewer Group","text"),("due_date","Feedback Due Date","text"),("feedback_mode","Feedback Mode","text"),("included_sections","Included Sections","list"),("questions","Reviewer Questions","list"),("tags","Tags","list")],
            "# Reviewer Package\n\n## Purpose\n\n## Intended Reviewers\n\n## Manuscript Version\n\n## Included Material\n\n## Excluded Material\n\n## Reading Instructions\n\n## Questions for Reviewers\n\n## Feedback Guidelines\n\n## Due Date / Review Window\n\n## Viewer / Delivery Notes\n\n## Returned Feedback\n\n## Package History",
        ),
        built_in_scoped(
            "review-revision-checklist",
            "Revision Checklist",
            "review",
            "revision",
            "checklist",
            None,
            "Reusable or project-specific checklist for verifying revision work across structure, scenes, continuity, prose, and final cleanup.",
            &["status", "revision_round", "manuscript_version", "checklist_type", "tags"],
            &[("status","Status","text"),("revision_round","Revision Round","text"),("manuscript_version","Manuscript Version","text"),("checklist_type","Checklist Type","text"),("focus_areas","Focus Areas","list"),("tags","Tags","list")],
            "# Revision Checklist\n\n## Structure\n\n- [ ] Major structural issues addressed\n\n## Plot & Causality\n\n- [ ] Cause and effect remain clear\n\n## Character\n\n- [ ] Character motivations and arcs remain coherent\n\n## Scene Function\n\n- [ ] Every retained scene advances story, character, or necessary context\n\n## Pacing & Tension\n\n- [ ] Pacing matches intended story pressure\n\n## Continuity\n\n- [ ] Timeline, locations, knowledge, possessions, and relationships are consistent\n\n## Worldbuilding & Setting\n\n- [ ] Canon details remain consistent\n\n## Prose & Dialogue\n\n- [ ] Recurring prose issues addressed\n\n## Final Verification\n\n- [ ] Revised manuscript reviewed after changes",
        ),
    ]
}

fn outline_templates() -> Vec<EntityTemplate> {
    vec![
        built_in_scoped(
            "outline-act",
            "Act",
            "outline",
            "structure",
            "note",
            Some("act"),
            "High-level story act for goals, turning points, pressure, character movement, and structural purpose.",
            &["act_number", "structure_role", "status", "start_point", "end_point", "tags"],
            &[("act_number","Act Number / Label","text"),("structure_role","Structural Role","text"),("status","Status","text"),("start_point","Opening Story Point","text"),("end_point","Closing Story Point","text"),("word_target","Word Target","number"),("pov_focus","POV Focus","list"),("plot_threads","Plot Threads","list"),("tags","Tags","list")],
            "# Act Summary\n\n## Structural Purpose\n\n## Opening State\n\n## Primary Goals & Questions\n\n## Major Plot Movement\n\n## Character Movement\n\n## Escalation & Complications\n\n## Key Reveals & Decisions\n\n## Turning Point\n\n## Closing State\n\n## Chapters & Scenes\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "outline-plot-point",
            "Plot Point",
            "outline",
            "structure",
            "event",
            Some("plot-point"),
            "Major structural story event such as an inciting incident, midpoint, crisis, climax, reversal, or threshold.",
            &["plot_point_type", "status", "act", "chapter", "scene", "story_effect", "tags"],
            &[("plot_point_type","Plot Point Type","text"),("status","Status","text"),("act","Act","text"),("chapter","Chapter","text"),("scene","Scene","text"),("story_effect","Story Effect","multiline"),("characters","Characters","list"),("plot_threads","Plot Threads","list"),("tags","Tags","list")],
            "# Plot Point Summary\n\n## Structural Function\n\n## Setup\n\n## Trigger / Event\n\n## Immediate Consequence\n\n## Character Decision\n\n## Stakes & Pressure\n\n## Plot Threads Affected\n\n## Information Revealed or Withheld\n\n## New Story Direction\n\n## Linked Chapters & Scenes\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "outline-beat",
            "Beat",
            "outline",
            "planning",
            "beat",
            None,
            "Atomic story beat for action, reaction, information, emotion, decision, or transition.",
            &["beat_type", "status", "chapter", "scene", "pov", "purpose", "tags"],
            &[("beat_type","Beat Type","text"),("status","Status","text"),("chapter","Chapter","text"),("scene","Scene","text"),("pov","POV Character","text"),("purpose","Purpose","text"),("characters","Characters","list"),("location","Location","text"),("plot_threads","Plot Threads","list"),("tags","Tags","list")],
            "# Beat Summary\n\n## Purpose\n\n## Setup\n\n## Action / Information\n\n## Character Reaction\n\n## Change Produced\n\n## Transition\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "outline-scene-card",
            "Scene Card",
            "outline",
            "planning",
            "scene",
            Some("scene-card"),
            "Plottr-style scene planning card linked to manuscript scene identity rather than a competing prose document.",
            &["status", "chapter", "pov", "location", "story_date", "scene_purpose", "conflict", "outcome", "tags"],
            &[("status","Status","text"),("chapter","Chapter","text"),("pov","POV Character","text"),("location","Location","text"),("story_date","Story Date / Time","text"),("scene_purpose","Scene Purpose","text"),("conflict","Conflict","text"),("outcome","Outcome / Change","text"),("characters","Characters Present","list"),("plot_threads","Plot Threads","list"),("word_target","Word Target","number"),("tags","Tags","list")],
            "# Scene Card\n\n## Scene Purpose\n\n## Opening Situation\n\n## POV Goal\n\n## Conflict & Opposition\n\n## Escalation\n\n## Key Information / Reveal\n\n## Emotional Movement\n\n## Decision / Turning Point\n\n## Outcome & Change\n\n## Exit / Hook\n\n## Characters & Location\n\n## Plot Threads\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "outline-chapter",
            "Chapter Outline",
            "outline",
            "planning",
            "chapter",
            Some("chapter-outline"),
            "Chapter-level planning document for purpose, scene sequence, pacing, POV, reveals, and chapter-level change.",
            &["chapter_number", "status", "pov", "chapter_purpose", "opening_state", "closing_state", "tags"],
            &[("chapter_number","Chapter Number / Label","text"),("status","Status","text"),("pov","Primary POV","text"),("chapter_purpose","Chapter Purpose","text"),("opening_state","Opening State","text"),("closing_state","Closing State","text"),("story_date","Story Date / Range","text"),("locations","Locations","list"),("plot_threads","Plot Threads","list"),("word_target","Word Target","number"),("tags","Tags","list")],
            "# Chapter Outline\n\n## Chapter Purpose\n\n## Opening State\n\n## Scene Sequence\n\n## POV & Character Movement\n\n## Conflict & Escalation\n\n## Reveals, Clues & Information\n\n## Plot Threads Advanced\n\n## Pacing Notes\n\n## Chapter Turning Point\n\n## Closing State / Hook\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "outline-plot-thread",
            "Plot / Subplot Thread",
            "outline",
            "tracking",
            "plot-thread",
            None,
            "Tracks a main plot, subplot, mystery, romance, character, thematic, or other narrative thread across the manuscript.",
            &["thread_type", "status", "priority", "introduced", "resolved", "story_question", "tags"],
            &[("thread_type","Thread Type","text"),("status","Status","text"),("priority","Priority","text"),("introduced","Introduced","text"),("resolved","Resolved","text"),("story_question","Central Story Question","multiline"),("characters","Characters","list"),("locations","Locations","list"),("related_threads","Related Threads","list"),("tags","Tags","list")],
            "# Plot Thread Summary\n\n## Central Story Question\n\n## Setup / Introduction\n\n## Stakes\n\n## Characters Involved\n\n## Progression\n\n### Establishment\n\n### Development\n\n### Complication\n\n### Crisis / Turning Point\n\n### Resolution / Payoff\n\n## Clues, Reveals & Foreshadowing\n\n## Intersections With Other Threads\n\n## Chapter / Scene Tracking\n\n## Unresolved Questions\n\n## Continuity Notes",
        ),
        built_in_scoped(
            "outline-story-structure",
            "Story Structure",
            "outline",
            "framework",
            "note",
            Some("story-structure"),
            "Reusable structural framework for Three-Act, Save the Cat, Hero's Journey, custom beat sheets, or other planning methods.",
            &["framework", "status", "premise", "theme", "ending", "tags"],
            &[("framework","Framework / Method","text"),("status","Status","text"),("premise","Premise","multiline"),("theme","Theme / Controlling Idea","multiline"),("ending","Ending / Final State","multiline"),("target_words","Target Word Count","number"),("pov_characters","POV Characters","list"),("plot_threads","Plot Threads","list"),("tags","Tags","list")],
            "# Story Structure\n\n## Framework\n\n## Premise\n\n## Theme / Controlling Idea\n\n## Opening State\n\n## Structural Beats\n\n## Major Turning Points\n\n## Character Arcs\n\n## Plot & Subplot Threads\n\n## Midpoint / Central Shift\n\n## Crisis\n\n## Climax\n\n## Resolution / Final State\n\n## Pacing & Balance Notes\n\n## Custom Structure Notes",
        ),
    ]
}

fn manuscript_templates() -> Vec<EntityTemplate> {
    vec![
        built_in_scoped(
            "manuscript-chapter",
            "Chapter",
            "manuscript",
            "chapter",
            "chapter",
            None,
            "Standard manuscript chapter with lightweight production metadata and an uncluttered drafting body.",
            &["chapter_number", "pov", "status", "story_date", "location", "word_target", "tags"],
            &[("chapter_number","Chapter Number","text"),("pov","POV Character","text"),("status","Draft Status","text"),("story_date","Story Date / Time","text"),("location","Primary Location","text"),("word_target","Word Target","number"),("timeline","Timeline / Sequence","text"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-scene",
            "Scene",
            "manuscript",
            "scene",
            "scene",
            None,
            "Scene manuscript unit with POV, chronology, location, purpose, status, and drafting metadata.",
            &["pov", "status", "story_date", "location", "scene_purpose", "word_target", "tags"],
            &[("pov","POV Character","text"),("status","Draft Status","text"),("story_date","Story Date / Time","text"),("location","Location","text"),("scene_purpose","Scene Purpose","text"),("word_target","Word Target","number"),("timeline","Timeline / Sequence","text"),("characters","Characters Present","list"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-prologue",
            "Prologue",
            "manuscript",
            "chapter",
            "prologue",
            None,
            "Opening manuscript section that precedes the numbered chapter sequence.",
            &["pov", "status", "story_date", "location", "word_target", "tags"],
            &[("pov","POV Character","text"),("status","Draft Status","text"),("story_date","Story Date / Time","text"),("location","Primary Location","text"),("word_target","Word Target","number"),("timeline","Timeline / Sequence","text"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-epilogue",
            "Epilogue",
            "manuscript",
            "chapter",
            "epilogue",
            None,
            "Closing manuscript section after the main chapter sequence.",
            &["pov", "status", "story_date", "location", "word_target", "tags"],
            &[("pov","POV Character","text"),("status","Draft Status","text"),("story_date","Story Date / Time","text"),("location","Primary Location","text"),("word_target","Word Target","number"),("timeline","Timeline / Sequence","text"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-interlude",
            "Interlude",
            "manuscript",
            "chapter",
            "interlude",
            None,
            "Between-chapter manuscript section for alternate viewpoint, time, place, document, or narrative bridge.",
            &["pov", "status", "story_date", "location", "interlude_type", "word_target", "tags"],
            &[("pov","POV / Voice","text"),("status","Draft Status","text"),("story_date","Story Date / Time","text"),("location","Location","text"),("interlude_type","Interlude Type","text"),("word_target","Word Target","number"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-part-divider",
            "Part / Act Divider",
            "manuscript",
            "chapter",
            "part-divider",
            None,
            "Structural manuscript divider for parts or acts without forcing prose into the divider itself.",
            &["part_number", "subtitle", "status", "tags"],
            &[("part_number","Part / Act Number","text"),("subtitle","Part / Act Title","text"),("status","Status","text"),("epigraph","Epigraph","multiline"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-front-matter",
            "Front Matter",
            "manuscript",
            "chapter",
            "front-matter",
            None,
            "Front-matter manuscript unit for title pages, copyright, dedication, epigraphs, contents, forewords, and similar material.",
            &["matter_type", "status", "include_in_export", "tags"],
            &[("matter_type","Front Matter Type","text"),("status","Status","text"),("include_in_export","Include in Export","boolean"),("tags","Tags","list")],
            "",
        ),
        built_in_scoped(
            "manuscript-back-matter",
            "Back Matter",
            "manuscript",
            "chapter",
            "back-matter",
            None,
            "Back-matter manuscript unit for acknowledgments, appendices, glossaries, notes, author material, and similar content.",
            &["matter_type", "status", "include_in_export", "tags"],
            &[("matter_type","Back Matter Type","text"),("status","Status","text"),("include_in_export","Include in Export","boolean"),("tags","Tags","list")],
            "",
        ),
    ]
}

fn campaign_templates() -> Vec<EntityTemplate> {
    const VIS: (&str, &str, &str) = ("visibility", "Visibility", "text");
    vec![
        built_in(
            "campaign-overview",
            "Campaign Overview",
            "campaign",
            "note",
            Some("campaign-overview"),
            "The campaign's premise, tone and central conflict at a glance.",
            &["subtitle", "genre", "tone", "central_conflict", "start_date", "gm_name", "status"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("genre","Genre","text"),("tone","Tone / Theme","text"),("central_conflict","Central Conflict","multiline"),("start_date","Campaign Start Date","text"),("gm_name","Game Master","text"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Premise\n\n## Tone & Expectations\n\n## Central Conflict\n\n## Major Threads\n\n## Player-Facing Pitch\n\n## GM Notes",
        ),
        built_in(
            "npc-profile",
            "NPC Profile",
            "character",
            "character",
            Some("npc"),
            "Non-player character: role, disposition, motivation and secrets. No combat statistics required.",
            &["subtitle", "disposition", "role", "affiliation", "motivation", "current_location", "status", "visibility"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("disposition","Disposition","text"),("role","Role","text"),("affiliation","Affiliation","text"),("motivation","Motivation","multiline"),("current_location","Current Location","text"),("status","Status","text"),("aliases","Aliases","list"),("relationships","Relationships","list"),VIS,("tags","Tags","list")],
            "## Appearance\n\n## Personality & Mannerisms\n\n## Wants & Motivation\n\n## Secrets\n\n## Relationships\n\n## Voice & Quotes\n\n## Statistics Reference\n\n## Session History",
        ),
        built_in(
            "player-character-placeholder",
            "Player Character (Placeholder)",
            "character",
            "character",
            Some("player-character"),
            "Minimal placeholder for a player character until Character Studio is available. Not a character sheet.",
            &["player", "status", "character_record"],
            &[("image","Image","image"),("player","Player","text"),("status","Status","text"),("character_record","Character Record","text"),VIS,("tags","Tags","list")],
            "## Concept\n\n## Background Hooks\n\n## Party Ties\n\n## GM Notes",
        ),
        built_in(
            "creature-profile",
            "Creature Profile",
            "creature",
            "note",
            Some("creature"),
            "Monster or creature profile with habitat, behaviour and encounter notes; rules statistics stay optional.",
            &["subtitle", "creature_type", "habitat", "threat", "status", "visibility"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("creature_type","Creature Type","text"),("habitat","Habitat","text"),("threat","Threat / Challenge","text"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Description\n\n## Behaviour & Tactics\n\n## Habitat & Ecology\n\n## Lore\n\n## Statistics Reference\n\n## Loot & Rewards",
        ),
        built_in(
            "quest-adventure",
            "Quest / Adventure",
            "adventure",
            "plot-thread",
            Some("quest"),
            "Adventure, story arc or quest with objectives, branches, rewards and consequences.",
            &["subtitle", "quest_type", "status", "patron", "objective", "reward", "visibility"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("quest_type","Quest Type","text"),("status","Status","text"),("patron","Patron / Quest Giver","text"),("objective","Objective","multiline"),("reward","Reward","text"),VIS,("tags","Tags","list")],
            "## Hook\n\n## Objectives\n\n## Key Locations\n\n## Key NPCs\n\n## Branches & Choices\n\n## Complications\n\n## Rewards\n\n## Consequences",
        ),
        built_in(
            "encounter-card",
            "Encounter",
            "adventure",
            "note",
            Some("encounter"),
            "Combat, social or exploration encounter linked to characters, creatures, maps and rewards.",
            &["subtitle", "encounter_type", "location", "difficulty", "status", "visibility"],
            &[("image","Image","image"),("subtitle","Subtitle","text"),("encounter_type","Encounter Type","text"),("location","Location","text"),("difficulty","Difficulty","text"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Setup\n\n## Participants\n\n## Terrain & Features\n\n## Tactics or Goals\n\n## Scaling\n\n## Outcome & Rewards",
        ),
        built_in(
            "session-plan",
            "Session Plan",
            "campaign",
            "note",
            Some("session-plan"),
            "Prep sheet for an upcoming game session.",
            &["session_number", "session_date", "status", "visibility"],
            &[("image","Image","image"),("session_number","Session Number","number"),("session_date","Session Date","text"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Recap\n\n## Strong Start\n\n## Potential Scenes\n\n## Secrets & Clues\n\n## NPCs\n\n## Locations\n\n## Encounters\n\n## Rewards",
        ),
        built_in(
            "session-log",
            "Session Log",
            "campaign",
            "note",
            Some("session-log"),
            "Chronological record of what happened at the table.",
            &["session_number", "session_date", "attendance", "visibility"],
            &[("image","Image","image"),("session_number","Session Number","number"),("session_date","Session Date","text"),("attendance","Attendance","list"),VIS,("tags","Tags","list")],
            "## Summary\n\n## Key Events\n\n## Player Discoveries\n\n## Loot & Rewards\n\n## Campaign State Changes\n\n## Open Threads",
        ),
        built_in(
            "house-rules",
            "House Rules",
            "campaign",
            "note",
            Some("house-rules"),
            "Table rules, variants and rulings for this campaign.",
            &["rules_system", "status"],
            &[("image","Image","image"),("rules_system","Rules System","text"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Rules Variants\n\n## Character Creation\n\n## Rests, Death & Downtime\n\n## Rulings Log",
        ),
        built_in(
            "gm-notes",
            "GM Notes",
            "campaign",
            "note",
            Some("gm-notes"),
            "Private Game Master notes, secrets and reminders.",
            &["status", "visibility"],
            &[("image","Image","image"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Notes\n\n## Secrets & Clues\n\n## Reminders",
        ),
        built_in(
            "map-reference",
            "Map Reference",
            "campaign",
            "note",
            Some("map-reference"),
            "References to existing map files. Map Studio does not generate these yet.",
            &["map_scale", "status"],
            &[("image","Image","image"),("map_scale","Scale","text"),("status","Status","text"),VIS,("tags","Tags","list")],
            "## Map Files\n\n## Key\n\n## Notes",
        ),
        built_in(
            "gm-guide",
            "GM Guide",
            "campaign",
            "note",
            Some("gm-guide"),
            "Offline Game Master advice. Catalyst's starter guides are original and editable.",
            &["guide_topic", "source", "license"],
            &[("image","Image","image"),("guide_topic","Topic","text"),("source","Source","text"),("license","License","text"),VIS,("tags","Tags","list")],
            "",
        ),
        built_in(
            "random-table",
            "Random Table",
            "campaign",
            "note",
            Some("random-table"),
            "A rollable table. Roll the listed die and read the matching row.",
            &["die", "table_category", "source"],
            &[("image","Image","image"),("die","Die","text"),("table_category","Category","text"),("source","Source","text"),("license","License","text"),VIS,("tags","Tags","list")],
            "",
        ),
    ]
}

/// Old LithoMark templates remain resolvable by ID for existing documents,
/// but are deliberately absent from new Catalyst document creation menus.
fn legacy_lithomark_templates() -> Vec<EntityTemplate> {
    let mut result = Vec::new();
    result.extend(manuscript_templates());
    result.extend(outline_templates());
    result.extend(review_templates());
    result.extend(publishing_templates());
    result.extend(project_templates());
    result
}


/// RPG additions use the existing file-backed entity registry and do not create
/// parallel campaign or character mechanics engines. Identifiers are stable.
fn catalyst_additional_templates() -> Vec<EntityTemplate> {
    vec![
        built_in(
            "adventure-overview", "Adventure Overview", "adventure", "adventure", Some("adventure"),
            "Adventure summary, hooks and playable branches.",
            &["premise", "status", "starting_location", "goal", "level_range"],
            &[("image", "Image", "image"), ("premise", "Premise", "text"), ("status", "Status", "text"), ("starting_location", "Starting Location", "text"), ("goal", "Goal", "text"), ("level_range", "Level Range", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Premise\n\n## Player Hook\n\n## Objectives\n\n## Scene Sequence\n\n## Key NPCs and Factions\n\n## Locations and Maps\n\n## Branches and Consequences\n\n## GM Secrets\n\n## Rewards",
        ),
        built_in(
            "adventure-scene", "Adventure Scene", "adventure", "adventure", Some("scene"),
            "Scene with triggers, choices and outcomes.",
            &["scene_type", "location", "objective", "participants", "status"],
            &[("image", "Image", "image"), ("scene_type", "Scene Type", "text"), ("location", "Location", "text"), ("objective", "Objective", "text"), ("participants", "Participants", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Read-Aloud Introduction\n\n## What Players Can Learn\n\n## NPCs and Creatures\n\n## Choices and Complications\n\n## Encounter or Map Link\n\n## Possible Outcomes\n\n## GM Notes",
        ),
        built_in(
            "quest-chain", "Quest Chain", "adventure", "adventure", Some("quest-chain"),
            "Linked quests, prerequisites and consequences.",
            &["status", "starting_quest", "goal", "reward"],
            &[("image", "Image", "image"), ("status", "Status", "text"), ("starting_quest", "Starting Quest", "text"), ("goal", "Goal", "text"), ("reward", "Reward", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Quest Sequence\n\n## Prerequisites\n\n## Branch Points\n\n## Dependencies\n\n## Rewards\n\n## Consequences",
        ),
        built_in(
            "puzzle-profile", "Puzzle", "adventure", "adventure", Some("puzzle"),
            "Playable puzzle with clues and alternate solutions.",
            &["location", "difficulty", "solution", "status"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("difficulty", "Difficulty", "text"), ("solution", "Solution", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Setup\n\n## Player-Facing Clues\n\n## Solution\n\n## Hints\n\n## Alternative Approaches\n\n## Consequences",
        ),
        built_in(
            "trap-hazard", "Trap & Hazard", "adventure", "adventure", Some("hazard"),
            "Terrain or constructed hazard with ruleset optional.",
            &["location", "trigger", "effect", "detection", "difficulty"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("trigger", "Trigger", "text"), ("effect", "Effect", "text"), ("detection", "Detection", "text"), ("difficulty", "Difficulty", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Trigger\n\n## Detection and Warnings\n\n## Effects\n\n## Countermeasures\n\n## Ruleset Mechanics Reference",
        ),
        built_in(
            "secret-clue", "Secret & Clue", "gm", "note", Some("clue"),
            "Track hidden information and discovery.",
            &["source", "location", "revealed_to", "status"],
            &[("image", "Image", "image"), ("source", "Source", "text"), ("location", "Location", "text"), ("revealed_to", "Revealed To", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## GM-Only Truth\n\n## Clue Presentation\n\n## Discovery Conditions\n\n## Player Knowledge\n\n## Related Mysteries\n\n## Follow-Up",
        ),
        built_in(
            "handout", "Player Handout", "gm", "note", Some("handout"),
            "Player-facing document or artifact.",
            &["audience", "status", "related_location"],
            &[("image", "Image", "image"), ("audience", "Audience", "text"), ("status", "Status", "text"), ("related_location", "Related Location", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Handout Content\n\n## When to Distribute\n\n## GM Notes",
        ),
        built_in(
            "loot-treasure", "Loot & Treasure", "item", "object", Some("loot"),
            "Treasure distribution and inventory references.",
            &["location", "owner", "value", "status"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("owner", "Owner", "text"), ("value", "Value", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Contents\n\n## Discovery and Ownership\n\n## Distribution\n\n## Ruleset Item References\n\n## Notes",
        ),
        built_in(
            "campaign-event", "Campaign State Event", "campaign", "event", Some("campaign-event"),
            "Record events that change ongoing campaign state.",
            &["session", "location", "participants", "status"],
            &[("image", "Image", "image"), ("session", "Session", "text"), ("location", "Location", "text"), ("participants", "Participants", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## What Happened\n\n## Player Decisions\n\n## NPC and Faction Consequences\n\n## World Changes\n\n## Open Threads",
        ),
        built_in(
            "session-zero", "Session Zero & Table Agreement", "session", "note", Some("session-zero"),
            "Play expectations and safety agreements.",
            &["session_date", "play_mode", "status"],
            &[("image", "Image", "image"), ("session_date", "Session Date", "text"), ("play_mode", "Play Mode", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Campaign Pitch\n\n## Player Expectations\n\n## Lines and Veils\n\n## Table Safety and Respect\n\n## Scheduling and Attendance\n\n## House Rules\n\n## Agreement and Follow-Up",
        ),
        built_in(
            "nation-kingdom", "Kingdom & Nation", "location", "location", Some("nation"),
            "Nation, government, culture and political relationships.",
            &["capital", "leader", "government", "population", "region"],
            &[("image", "Image", "image"), ("capital", "Capital", "text"), ("leader", "Leader", "text"), ("government", "Government", "text"), ("population", "Population", "text"), ("region", "Region", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Overview\n\n## History\n\n## Territory and Borders\n\n## Government\n\n## Peoples and Cultures\n\n## Settlements\n\n## Allies and Rivals\n\n## Adventure Hooks",
        ),
        built_in(
            "deity-profile", "Deity & Divine Figure", "worldbuilding", "worldbuilding", Some("deity"),
            "Deities, faith and cultural influence.",
            &["pantheon", "domains", "followers", "symbols", "status"],
            &[("image", "Image", "image"), ("pantheon", "Pantheon", "text"), ("domains", "Domains", "text"), ("followers", "Followers", "text"), ("symbols", "Symbols", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Origin and Lore\n\n## Beliefs and Worship\n\n## Clergy and Temples\n\n## Relationships\n\n## Campaign Relevance",
        ),
        built_in(
            "economy-currency", "Economy & Currency", "worldbuilding", "worldbuilding", Some("economy"),
            "Local economies and system-specific currencies.",
            &["region", "currency", "trade", "status"],
            &[("image", "Image", "image"), ("region", "Region", "text"), ("currency", "Currency", "text"), ("trade", "Trade", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Currency and Exchange\n\n## Trade Goods\n\n## Markets and Professions\n\n## Scarcity and Resources\n\n## Ruleset Reference",
        ),
        built_in(
            "calendar-system", "Calendar & Timekeeping", "worldbuilding", "worldbuilding", Some("calendar"),
            "Campaign calendars, festivals and era names.",
            &["era", "year_length", "calendar_type", "region"],
            &[("image", "Image", "image"), ("era", "Era", "text"), ("year_length", "Year Length", "text"), ("calendar_type", "Calendar Type", "text"), ("region", "Region", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Time Units\n\n## Months and Seasons\n\n## Holidays\n\n## Historical Eras\n\n## Campaign Dates",
        ),
        built_in(
            "flora-plant", "Plant & Flora", "worldbuilding", "worldbuilding", Some("flora"),
            "Fantasy or realistic flora with lore and uses.",
            &["habitat", "rarity", "properties", "hazards"],
            &[("image", "Image", "image"), ("habitat", "Habitat", "text"), ("rarity", "Rarity", "text"), ("properties", "Properties", "text"), ("hazards", "Hazards", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Habitat\n\n## Harvest and Uses\n\n## Hazards\n\n## Lore\n\n## Ruleset Reference",
        ),
        built_in(
            "tavern-shop", "Tavern, Inn & Shop", "location", "location", Some("business"),
            "Establishment and its services.",
            &["location", "owner", "services", "status"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("owner", "Owner", "text"), ("services", "Services", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Overview\n\n## Rooms and Layout\n\n## NPCs\n\n## Goods and Services\n\n## Rumors\n\n## Events and Hooks",
        ),
        built_in(
            "dungeon-site", "Dungeon & Adventure Site", "location", "location", Some("dungeon"),
            "Explorable site with rooms and encounters.",
            &["region", "entrance", "site_type", "status"],
            &[("image", "Image", "image"), ("region", "Region", "text"), ("entrance", "Entrance", "text"), ("site_type", "Site Type", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Approach\n\n## Site Background\n\n## Rooms and Areas\n\n## Encounters\n\n## Traps and Hazards\n\n## Treasure\n\n## Maps\n\n## Consequences",
        ),
        built_in(
            "equipment-reference", "Equipment Reference", "item", "object", Some("equipment"),
            "Item lore and linked ruleset equipment definition.",
            &["item_type", "ruleset_ref", "weight", "value", "rarity"],
            &[("image", "Image", "image"), ("item_type", "Item Type", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("weight", "Weight", "text"), ("value", "Value", "text"), ("rarity", "Rarity", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Usage\n\n## Ruleset Item Reference\n\n## Location and Ownership\n\n## Campaign Notes",
        ),
        built_in(
            "consumable-reference", "Consumable Reference", "item", "object", Some("consumable"),
            "Consumables and their in-world effects.",
            &["item_type", "ruleset_ref", "charges", "rarity"],
            &[("image", "Image", "image"), ("item_type", "Item Type", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("charges", "Charges", "text"), ("rarity", "Rarity", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Availability\n\n## Usage\n\n## Ruleset Effect Reference\n\n## Campaign Notes",
        ),
        built_in(
            "spell-power-lore", "Spell & Power Lore", "worldbuilding", "worldbuilding", Some("spell-lore"),
            "Spell lore separate from mechanics.",
            &["tradition", "origin", "ruleset_ref", "availability"],
            &[("image", "Image", "image"), ("tradition", "Tradition", "text"), ("origin", "Origin", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("availability", "Availability", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Lore and Origins\n\n## How It Appears in the World\n\n## Restrictions\n\n## Ruleset Spell Reference",
        ),
        built_in(
            "crafting-recipe", "Crafting Recipe", "item", "object", Some("crafting"),
            "Ingredients, instructions and system-linked crafting.",
            &["product", "materials", "tools", "time", "ruleset_ref"],
            &[("image", "Image", "image"), ("product", "Product", "text"), ("materials", "Materials", "text"), ("tools", "Tools", "text"), ("time", "Time", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Requirements\n\n## Process\n\n## Outcome\n\n## Availability\n\n## Ruleset Crafting Reference",
        ),
        built_in(
            "gm-screen-reference", "GM Screen Reference", "gm", "note", Some("screen-reference"),
            "Local quick-reference sheet for a GM.",
            &["ruleset_ref", "topic", "status"],
            &[("image", "Image", "image"), ("ruleset_ref", "Ruleset Ref", "text"), ("topic", "Topic", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Quick Reference\n\n## Procedures\n\n## Tables\n\n## Ruleset Citations\n\n## Notes",
        ),
        built_in(
            "rpg-print-profile", "RPG Print Profile", "campaign", "note", Some("print-profile"),
            "Preset for character, map, handout and campaign report printing.",
            &["output_type", "paper_size", "orientation", "ruleset_ref"],
            &[("image", "Image", "image"), ("output_type", "Output Type", "text"), ("paper_size", "Paper Size", "text"), ("orientation", "Orientation", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Print Purpose\n\n## Included Sections\n\n## Layout and Paper\n\n## Ruleset Citations\n\n## Notes",
        ),
    ]
}

/// Old LithoMark templates remain resolvable by ID for existing documents,
/// but are deliberately absent from new Catalyst document creation menus.
fn legacy_lithomark_templates() -> Vec<EntityTemplate> {
    let mut result = Vec::new();
    result.extend(manuscript_templates());
    result.extend(outline_templates());
    result.extend(review_templates());
    result.extend(publishing_templates());
    result.extend(project_templates());
    result
}


/// RPG additions use the existing file-backed entity registry and do not create
/// parallel campaign or character mechanics engines. Identifiers are stable.
fn catalyst_additional_templates() -> Vec<EntityTemplate> {
    vec![
        built_in(
            "adventure-overview", "Adventure Overview", "adventure", "adventure", Some("adventure"),
            "Adventure summary, hooks and playable branches.",
            &["premise", "status", "starting_location", "goal", "level_range"],
            &[("image", "Image", "image"), ("premise", "Premise", "text"), ("status", "Status", "text"), ("starting_location", "Starting Location", "text"), ("goal", "Goal", "text"), ("level_range", "Level Range", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Premise\n\n## Player Hook\n\n## Objectives\n\n## Scene Sequence\n\n## Key NPCs and Factions\n\n## Locations and Maps\n\n## Branches and Consequences\n\n## GM Secrets\n\n## Rewards",
        ),
        built_in(
            "adventure-scene", "Adventure Scene", "adventure", "adventure", Some("scene"),
            "Scene with triggers, choices and outcomes.",
            &["scene_type", "location", "objective", "participants", "status"],
            &[("image", "Image", "image"), ("scene_type", "Scene Type", "text"), ("location", "Location", "text"), ("objective", "Objective", "text"), ("participants", "Participants", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Read-Aloud Introduction\n\n## What Players Can Learn\n\n## NPCs and Creatures\n\n## Choices and Complications\n\n## Encounter or Map Link\n\n## Possible Outcomes\n\n## GM Notes",
        ),
        built_in(
            "quest-chain", "Quest Chain", "adventure", "adventure", Some("quest-chain"),
            "Linked quests, prerequisites and consequences.",
            &["status", "starting_quest", "goal", "reward"],
            &[("image", "Image", "image"), ("status", "Status", "text"), ("starting_quest", "Starting Quest", "text"), ("goal", "Goal", "text"), ("reward", "Reward", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Quest Sequence\n\n## Prerequisites\n\n## Branch Points\n\n## Dependencies\n\n## Rewards\n\n## Consequences",
        ),
        built_in(
            "puzzle-profile", "Puzzle", "adventure", "adventure", Some("puzzle"),
            "Playable puzzle with clues and alternate solutions.",
            &["location", "difficulty", "solution", "status"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("difficulty", "Difficulty", "text"), ("solution", "Solution", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Setup\n\n## Player-Facing Clues\n\n## Solution\n\n## Hints\n\n## Alternative Approaches\n\n## Consequences",
        ),
        built_in(
            "trap-hazard", "Trap & Hazard", "adventure", "adventure", Some("hazard"),
            "Terrain or constructed hazard with ruleset optional.",
            &["location", "trigger", "effect", "detection", "difficulty"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("trigger", "Trigger", "text"), ("effect", "Effect", "text"), ("detection", "Detection", "text"), ("difficulty", "Difficulty", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Trigger\n\n## Detection and Warnings\n\n## Effects\n\n## Countermeasures\n\n## Ruleset Mechanics Reference",
        ),
        built_in(
            "secret-clue", "Secret & Clue", "gm", "note", Some("clue"),
            "Track hidden information and discovery.",
            &["source", "location", "revealed_to", "status"],
            &[("image", "Image", "image"), ("source", "Source", "text"), ("location", "Location", "text"), ("revealed_to", "Revealed To", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## GM-Only Truth\n\n## Clue Presentation\n\n## Discovery Conditions\n\n## Player Knowledge\n\n## Related Mysteries\n\n## Follow-Up",
        ),
        built_in(
            "handout", "Player Handout", "gm", "note", Some("handout"),
            "Player-facing document or artifact.",
            &["audience", "status", "related_location"],
            &[("image", "Image", "image"), ("audience", "Audience", "text"), ("status", "Status", "text"), ("related_location", "Related Location", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Handout Content\n\n## When to Distribute\n\n## GM Notes",
        ),
        built_in(
            "loot-treasure", "Loot & Treasure", "item", "object", Some("loot"),
            "Treasure distribution and inventory references.",
            &["location", "owner", "value", "status"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("owner", "Owner", "text"), ("value", "Value", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Contents\n\n## Discovery and Ownership\n\n## Distribution\n\n## Ruleset Item References\n\n## Notes",
        ),
        built_in(
            "campaign-event", "Campaign State Event", "campaign", "event", Some("campaign-event"),
            "Record events that change ongoing campaign state.",
            &["session", "location", "participants", "status"],
            &[("image", "Image", "image"), ("session", "Session", "text"), ("location", "Location", "text"), ("participants", "Participants", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## What Happened\n\n## Player Decisions\n\n## NPC and Faction Consequences\n\n## World Changes\n\n## Open Threads",
        ),
        built_in(
            "session-zero", "Session Zero & Table Agreement", "session", "note", Some("session-zero"),
            "Play expectations and safety agreements.",
            &["session_date", "play_mode", "status"],
            &[("image", "Image", "image"), ("session_date", "Session Date", "text"), ("play_mode", "Play Mode", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Campaign Pitch\n\n## Player Expectations\n\n## Lines and Veils\n\n## Table Safety and Respect\n\n## Scheduling and Attendance\n\n## House Rules\n\n## Agreement and Follow-Up",
        ),
        built_in(
            "nation-kingdom", "Kingdom & Nation", "location", "location", Some("nation"),
            "Nation, government, culture and political relationships.",
            &["capital", "leader", "government", "population", "region"],
            &[("image", "Image", "image"), ("capital", "Capital", "text"), ("leader", "Leader", "text"), ("government", "Government", "text"), ("population", "Population", "text"), ("region", "Region", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Overview\n\n## History\n\n## Territory and Borders\n\n## Government\n\n## Peoples and Cultures\n\n## Settlements\n\n## Allies and Rivals\n\n## Adventure Hooks",
        ),
        built_in(
            "deity-profile", "Deity & Divine Figure", "worldbuilding", "worldbuilding", Some("deity"),
            "Deities, faith and cultural influence.",
            &["pantheon", "domains", "followers", "symbols", "status"],
            &[("image", "Image", "image"), ("pantheon", "Pantheon", "text"), ("domains", "Domains", "text"), ("followers", "Followers", "text"), ("symbols", "Symbols", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Origin and Lore\n\n## Beliefs and Worship\n\n## Clergy and Temples\n\n## Relationships\n\n## Campaign Relevance",
        ),
        built_in(
            "economy-currency", "Economy & Currency", "worldbuilding", "worldbuilding", Some("economy"),
            "Local economies and system-specific currencies.",
            &["region", "currency", "trade", "status"],
            &[("image", "Image", "image"), ("region", "Region", "text"), ("currency", "Currency", "text"), ("trade", "Trade", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Currency and Exchange\n\n## Trade Goods\n\n## Markets and Professions\n\n## Scarcity and Resources\n\n## Ruleset Reference",
        ),
        built_in(
            "calendar-system", "Calendar & Timekeeping", "worldbuilding", "worldbuilding", Some("calendar"),
            "Campaign calendars, festivals and era names.",
            &["era", "year_length", "calendar_type", "region"],
            &[("image", "Image", "image"), ("era", "Era", "text"), ("year_length", "Year Length", "text"), ("calendar_type", "Calendar Type", "text"), ("region", "Region", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Time Units\n\n## Months and Seasons\n\n## Holidays\n\n## Historical Eras\n\n## Campaign Dates",
        ),
        built_in(
            "flora-plant", "Plant & Flora", "worldbuilding", "worldbuilding", Some("flora"),
            "Fantasy or realistic flora with lore and uses.",
            &["habitat", "rarity", "properties", "hazards"],
            &[("image", "Image", "image"), ("habitat", "Habitat", "text"), ("rarity", "Rarity", "text"), ("properties", "Properties", "text"), ("hazards", "Hazards", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Habitat\n\n## Harvest and Uses\n\n## Hazards\n\n## Lore\n\n## Ruleset Reference",
        ),
        built_in(
            "tavern-shop", "Tavern, Inn & Shop", "location", "location", Some("business"),
            "Establishment and its services.",
            &["location", "owner", "services", "status"],
            &[("image", "Image", "image"), ("location", "Location", "text"), ("owner", "Owner", "text"), ("services", "Services", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Overview\n\n## Rooms and Layout\n\n## NPCs\n\n## Goods and Services\n\n## Rumors\n\n## Events and Hooks",
        ),
        built_in(
            "dungeon-site", "Dungeon & Adventure Site", "location", "location", Some("dungeon"),
            "Explorable site with rooms and encounters.",
            &["region", "entrance", "site_type", "status"],
            &[("image", "Image", "image"), ("region", "Region", "text"), ("entrance", "Entrance", "text"), ("site_type", "Site Type", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Approach\n\n## Site Background\n\n## Rooms and Areas\n\n## Encounters\n\n## Traps and Hazards\n\n## Treasure\n\n## Maps\n\n## Consequences",
        ),
        built_in(
            "equipment-reference", "Equipment Reference", "item", "object", Some("equipment"),
            "Item lore and linked ruleset equipment definition.",
            &["item_type", "ruleset_ref", "weight", "value", "rarity"],
            &[("image", "Image", "image"), ("item_type", "Item Type", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("weight", "Weight", "text"), ("value", "Value", "text"), ("rarity", "Rarity", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Usage\n\n## Ruleset Item Reference\n\n## Location and Ownership\n\n## Campaign Notes",
        ),
        built_in(
            "consumable-reference", "Consumable Reference", "item", "object", Some("consumable"),
            "Consumables and their in-world effects.",
            &["item_type", "ruleset_ref", "charges", "rarity"],
            &[("image", "Image", "image"), ("item_type", "Item Type", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("charges", "Charges", "text"), ("rarity", "Rarity", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Description\n\n## Availability\n\n## Usage\n\n## Ruleset Effect Reference\n\n## Campaign Notes",
        ),
        built_in(
            "spell-power-lore", "Spell & Power Lore", "worldbuilding", "worldbuilding", Some("spell-lore"),
            "Spell lore separate from mechanics.",
            &["tradition", "origin", "ruleset_ref", "availability"],
            &[("image", "Image", "image"), ("tradition", "Tradition", "text"), ("origin", "Origin", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("availability", "Availability", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Lore and Origins\n\n## How It Appears in the World\n\n## Restrictions\n\n## Ruleset Spell Reference",
        ),
        built_in(
            "crafting-recipe", "Crafting Recipe", "item", "object", Some("crafting"),
            "Ingredients, instructions and system-linked crafting.",
            &["product", "materials", "tools", "time", "ruleset_ref"],
            &[("image", "Image", "image"), ("product", "Product", "text"), ("materials", "Materials", "text"), ("tools", "Tools", "text"), ("time", "Time", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Requirements\n\n## Process\n\n## Outcome\n\n## Availability\n\n## Ruleset Crafting Reference",
        ),
        built_in(
            "gm-screen-reference", "GM Screen Reference", "gm", "note", Some("screen-reference"),
            "Local quick-reference sheet for a GM.",
            &["ruleset_ref", "topic", "status"],
            &[("image", "Image", "image"), ("ruleset_ref", "Ruleset Ref", "text"), ("topic", "Topic", "text"), ("status", "Status", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Quick Reference\n\n## Procedures\n\n## Tables\n\n## Ruleset Citations\n\n## Notes",
        ),
        built_in(
            "rpg-print-profile", "RPG Print Profile", "campaign", "note", Some("print-profile"),
            "Preset for character, map, handout and campaign report printing.",
            &["output_type", "paper_size", "orientation", "ruleset_ref"],
            &[("image", "Image", "image"), ("output_type", "Output Type", "text"), ("paper_size", "Paper Size", "text"), ("orientation", "Orientation", "text"), ("ruleset_ref", "Ruleset Ref", "text"), ("visibility", "Visibility", "text"), ("tags", "Tags", "list")],
            "## Print Purpose\n\n## Included Sections\n\n## Layout and Paper\n\n## Ruleset Citations\n\n## Notes",
        ),
    ]
}

fn built_in_scoped(
    id: &str,
    name: &str,
    scope: &str,
    category: &str,
    document_type: &str,
    subtype: Option<&str>,
    description: &str,
    overview_fields: &[&str],
    fields: &[(&str, &str, &str)],
    body: &str,
) -> EntityTemplate {
    let mut template = built_in(
        id,
        name,
        category,
        document_type,
        subtype,
        description,
        overview_fields,
        fields,
        body,
    );
    template.manifest.scope = scope.into();
    template
}

fn built_in(
    id: &str,
    name: &str,
    category: &str,
    document_type: &str,
    subtype: Option<&str>,
    description: &str,
    overview_fields: &[&str],
    fields: &[(&str, &str, &str)],
    body: &str,
) -> EntityTemplate {
    EntityTemplate {
        manifest: EntityTemplateManifest {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            scope: "story-development".into(),
            document_type: document_type.into(),
            subtype: subtype.map(str::to_string),
            version: 1,
            description: Some(description.into()),
            overview_fields: overview_fields.iter().map(|value| (*value).to_string()).collect(),
            image_field: "image".into(),
            fields: fields
                .iter()
                .map(|(key, label, field_type)| EntityTemplateField {
                    key: (*key).into(),
                    label: (*label).into(),
                    field_type: (*field_type).into(),
                    required: false,
                    options: Vec::new(),
                })
                .collect(),
        },
        body: body.into(),
        source: "built-in".into(),
        relative_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_built_ins_and_recursively_discovers_user_templates() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Templates");
        let nested = root.join("Organizations");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("Academy.md"),
            r#"---
lithomark_template:
  id: academy-profile
  name: Academy Profile
  category: organization
  document_type: organization
  subtype: academy
  version: 1
  fields:
    - key: image
      label: Image
      type: image
    - key: headmaster
      label: Headmaster
      type: text
---

# Overview

## Faculty
"#,
        )
        .unwrap();

        let registry = load_registry(&root).unwrap();

        assert!(registry
            .templates
            .iter()
            .any(|template| template.manifest.id == "character-profile"));
        let academy = registry
            .templates
            .iter()
            .find(|template| template.manifest.id == "academy-profile")
            .unwrap();
        assert_eq!(academy.source, "user");
        assert_eq!(
            academy.relative_path.as_deref(),
            Some("Organizations/Academy.md")
        );
        assert!(registry.issues.is_empty());
    }

    #[test]
    fn duplicate_user_id_cannot_override_built_in() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Templates");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("Override.md"),
            r#"---
lithomark_template:
  id: character-profile
  name: Replacement Character
  category: character
  document_type: character
  version: 1
---
"#,
        )
        .unwrap();

        let registry = load_registry(&root).unwrap();

        assert_eq!(
            registry
                .templates
                .iter()
                .filter(|template| template.manifest.id == "character-profile")
                .count(),
            1
        );
        assert_eq!(registry.issues.len(), 1);
        assert!(registry.issues[0].message.contains("already registered"));
    }

    #[test]
    fn invalid_template_is_reported_without_hiding_valid_templates() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Templates");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Broken.md"), "not frontmatter").unwrap();

        let registry = load_registry(&root).unwrap();

        assert!(!registry.templates.is_empty());
        assert_eq!(registry.issues.len(), 1);
        assert!(registry.issues[0].relative_path.ends_with("Broken.md"));
    }

    #[test]
    fn creates_missing_templates_folder() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Templates");

        let registry = load_registry(&root).unwrap();

        assert!(root.is_dir());
        assert_eq!(registry.user_templates_dir, root.to_string_lossy());
    }

    #[test]
    fn rejects_unknown_overview_and_image_fields() {
        let mut manifest = built_in_templates()[0].manifest.clone();
        manifest.overview_fields.push("missing".into());
        assert!(validate_manifest(&manifest).unwrap_err().contains("Overview field"));

        manifest.overview_fields.pop();
        manifest.image_field = "missing_image".into();
        assert!(validate_manifest(&manifest).unwrap_err().contains("Image field"));
    }

    #[test]
    fn saves_edits_and_deletes_user_templates() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Templates");
        fs::create_dir_all(&root).unwrap();

        let manifest = EntityTemplateManifest {
            id: "custom-character".into(),
            name: "Custom Character".into(),
            category: "character".into(),
            scope: "story-development".into(),
            document_type: "character".into(),
            subtype: None,
            version: 1,
            description: Some("Synthetic custom template.".into()),
            overview_fields: vec!["role".into()],
            image_field: "image".into(),
            fields: vec![
                EntityTemplateField {
                    key: "image".into(),
                    label: "Image".into(),
                    field_type: "image".into(),
                    required: false,
                    options: vec![],
                },
                EntityTemplateField {
                    key: "role".into(),
                    label: "Role".into(),
                    field_type: "text".into(),
                    required: false,
                    options: vec![],
                },
            ],
        };

        let registry = save_user_template(
            &root,
            SaveUserEntityTemplateInput {
                manifest: manifest.clone(),
                body: "# Custom Character\n\n## Notes".into(),
                relative_path: "Characters/Custom Character.md".into(),
                previous_relative_path: None,
            },
        )
        .unwrap();
        assert!(registry.templates.iter().any(|template| template.manifest.id == "custom-character"));

        let registry = save_user_template(
            &root,
            SaveUserEntityTemplateInput {
                manifest,
                body: "# Custom Character\n\n## Revised Notes".into(),
                relative_path: "Characters/Revised Character.md".into(),
                previous_relative_path: Some("Characters/Custom Character.md".into()),
            },
        )
        .unwrap();
        assert!(!root.join("Characters/Custom Character.md").exists());
        assert!(root.join("Characters/Revised Character.md").exists());
        assert!(registry.templates.iter().any(|template| template.relative_path.as_deref() == Some("Characters/Revised Character.md")));

        let registry = delete_user_template(&root, "Characters/Revised Character.md").unwrap();
        assert!(!registry.templates.iter().any(|template| template.manifest.id == "custom-character"));
    }

    #[test]
    fn refuses_to_replace_built_in_template_id() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Templates");
        fs::create_dir_all(&root).unwrap();
        let manifest = built_in_templates()[0].manifest.clone();

        let error = save_user_template(
            &root,
            SaveUserEntityTemplateInput {
                manifest,
                body: "# Override".into(),
                relative_path: "Override.md".into(),
                previous_relative_path: None,
            },
        )
        .unwrap_err();

        assert!(error.contains("built-in"));
    }
}

#[cfg(test)]
mod catalyst_template_catalog_tests {
    use super::*;
    #[test]
    fn novel_only_templates_are_retired_but_legacy_ids_resolve() {
        let root = tempfile::tempdir().unwrap();
        let registry = load_registry(&root.path().join("Templates")).unwrap();
        assert!(!registry.templates.iter().any(|t| t.manifest.id == "project-novel"));
        assert!(!registry.templates.iter().any(|t| t.manifest.id == "manuscript-chapter"));
        assert!(built_in_template("project-novel").is_some());
        assert!(built_in_template("character-profile").is_some());
    }
    #[test]
    fn rpg_templates_have_unique_ids_and_valid_manifests() {
        let root = tempfile::tempdir().unwrap();
        let registry = load_registry(&root.path().join("Templates")).unwrap();
        let mut ids = std::collections::BTreeSet::new();
        for t in &registry.templates {
            assert!(ids.insert(t.manifest.id.as_str()), "duplicate {}", t.manifest.id);
            validate_manifest(&t.manifest).unwrap();
        }
        for id in ["campaign-overview", "quest-adventure", "adventure-scene", "dungeon-site", "secret-clue", "rpg-print-profile"] {
            assert!(ids.contains(id), "missing {id}");
        }
    }
}
