//! Offline Game Master starter references (020 §9).
//!
//! Every guide and table here was written for Catalyst. They carry no
//! third-party text, so they can ship with the app and be copied into each new
//! campaign, where the GM can edit them freely. Anything from an outside source
//! must clear the 020 §9 licensing gate before it is added here.

pub const LIBRARY_SOURCE: &str = "Catalyst starter library";
pub const LIBRARY_LICENSE: &str = "Original Catalyst content; free to edit and reuse in your games";

pub const GUIDES_FOLDER: &str = "Game Master/Guides";
pub const TABLES_FOLDER: &str = "Game Master/Random Tables";

pub struct Guide {
    pub title: &'static str,
    pub topic: &'static str,
    pub body: &'static str,
}

pub struct Table {
    pub title: &'static str,
    pub category: &'static str,
    pub rows: &'static [&'static str],
}

impl Table {
    pub fn die(&self) -> String {
        format!("d{}", self.rows.len())
    }

    pub fn body(&self) -> String {
        let mut out = format!(
            "Roll a {} and read the matching row. Edit or replace any entry to suit your table.\n\n| {} | Result |\n| --- | --- |\n",
            self.die(),
            self.die()
        );
        for (index, row) in self.rows.iter().enumerate() {
            out.push_str(&format!("| {} | {} |\n", index + 1, row));
        }
        out
    }
}

pub const GUIDES: &[Guide] = &[
    Guide {
        title: "Starting Your First Campaign",
        topic: "Getting started",
        body: r#"## Pick a system you can explain in two minutes
Choose rules your players can learn at the table. A light system lets you focus on the story while everyone finds their footing. You can always switch later.

## Run a Session Zero
Before the first adventure, meet as a group and agree on:
- **Tone.** Heroic, grim, comedic or somewhere in between.
- **Content lines and veils.** Topics that never appear (lines) and topics that happen off screen (veils). Revisit these whenever someone asks.
- **Schedule.** How often you play, how long a session runs, and what happens when someone misses one.
- **Character ties.** Each character should know at least one other character and have one reason to stay with the group.

## Start small
Begin in one town with one problem that needs solving tonight. Expand the world only where the players look. A single good village beats a continent of empty names.

## Checklist
- [ ] Session Zero notes saved in Sessions/Session Plans
- [ ] Each player character has a goal and a tie to the group
- [ ] First adventure has a clear hook and a strong opening scene"#,
    },
    Guide {
        title: "Planning Adventures and Quests",
        topic: "Campaign planning",
        body: r#"## Plan situations, not plots
Write down who wants what, what happens if nobody interferes, and where the players can step in. Let the players choose the path. Prepare the pieces they are likely to touch.

## The five-part adventure
1. **Hook.** Why the characters care, tied to a goal or a person they know.
2. **Complication.** The first plan fails or the problem is bigger than it looked.
3. **Choice.** At least two reasonable ways forward, each with a cost.
4. **Climax.** The opposition acts at full strength.
5. **Consequence.** The world changes because of what the party did.

## Branches and consequences
For every major choice, note one thing that improves and one thing that gets worse. Carry those changes into later sessions so the players feel their decisions matter.

## Secrets and clues
List eight to ten short secrets the players could learn. Attach each to whatever scene they actually reach, instead of hiding it behind one specific door."#,
    },
    Guide {
        title: "Building a Living World",
        topic: "Worldbuilding",
        body: r#"## Start from the table outward
Detail the place where play begins, then the region around it, then the wider world. Leave blank space on purpose so there is room to discover.

## Questions for any place
- Who is in charge, and who wants to be?
- What does everyone here need that is hard to get?
- What do people believe, and what do they fear?
- What changed recently?

## Factions
Give each faction a goal, a resource, a weakness and a current move. When the players rest or travel, advance each faction one step. The world then moves without them.

## Religion, magic and economy
Decide what ordinary people see of each. A farmer does not need cosmology, but they know which shrine blesses the harvest and what a healer charges."#,
    },
    Guide {
        title: "Running NPCs and Dialogue",
        topic: "NPCs",
        body: r#"## Three details are enough
Give an NPC one look, one want and one habit (a phrase, a gesture, a smell). Players remember a gruff ferryman who taps his oar far better than a page of history.

## Wants drive conversations
Every NPC wants something from the characters, even if it is only to be left alone. Decide what they offer, what they refuse and what would change their mind.

## Relationships
Track how each recurring NPC feels about the party: friendly, neutral, wary or hostile. Shift it one step after a meaningful kindness or slight.

## Improvised names
Keep the NPC Names table open during play. Note every name you invent in the Characters folder so continuity holds."#,
    },
    Guide {
        title: "Encounters: Combat, Social and Exploration",
        topic: "Encounters",
        body: r#"## Every encounter needs a reason
Ask what the encounter reveals, costs or changes. If the answer is nothing, summarize it in one sentence and move on.

## Combat
Give the opposition a goal other than "fight to the death": steal something, escape, hold a door. Use terrain such as height, cover and hazards. Let enemies retreat or surrender.

## Social
Set out what each side wants and what it will trade. Use rolls to decide how much the other side gives, not whether a good argument works at all.

## Exploration and environment
Weather, darkness, distance and time are opponents too. Present a clear danger, signs that warn of it, and more than one way past it.

## Dungeons, traps and puzzles
Telegraph traps with clues the players can notice. A puzzle should have several possible solutions, and a hint should be available when the group is stuck."#,
    },
    Guide {
        title: "Improvisation and Player Agency",
        topic: "Running the table",
        body: r#"## Say yes, or roll the dice
When a player tries something reasonable, either let it happen or ask for a roll with clear stakes. Saying no ends ideas, so save it for safety and table agreements.

## Failing forward
A failed roll should change the situation, not stop the story. The lock opens but the guard hears it, or the answer comes at a price.

## Continuity
After each session, spend five minutes on a Session Log: what happened, who they met, what they promised and what is still unresolved. Read the open threads aloud at the start of the next session.

## Pacing
When energy drops, cut to the next interesting scene or have an NPC act. When the table is engaged, let a scene breathe even if it was not planned."#,
    },
    Guide {
        title: "Dice, Odds and GM Checklists",
        topic: "Dice and procedures",
        body: r#"## Dice notation
- **2d6+1** means roll two six-sided dice, add them and add 1.
- **d%** or **d100** means roll a percentage.
- **Advantage / disadvantage** usually means roll twice and keep the higher or lower result.

## Rough odds
- A single d20 roll of 11 or more succeeds 50% of the time. Each +1 adds 5%.
- On 2d6, a 7 is the most common result (about 17%). An 8 or more comes up about 42% of the time.
- Rolling several dice and adding them clusters results near the middle. One large die spreads them evenly.

## Public or private rolls
Roll in the open when you want shared tension. Roll in private when the result would give away information the characters do not have yet. Catalyst's dice roller records both in Sessions/Roll Logs.

## Before the session
- [ ] Review the last Session Log and open threads
- [ ] Prepare a strong opening scene
- [ ] List secrets and clues
- [ ] Note key NPC names and motives
- [ ] Have random tables ready

## After the session
- [ ] Write the Session Log
- [ ] Advance factions one step
- [ ] Update NPC attitudes and campaign state"#,
    },
];

pub const TABLES: &[Table] = &[
    Table {
        title: "NPC Names",
        category: "NPCs",
        rows: &[
            "Ansel Rook", "Brielle Ashdown", "Corvin Hale", "Dara Quillfeather",
            "Edrin Mossgate", "Fenna Thorne", "Galen Pike", "Hesper Vane",
            "Ivo Brackwater", "Juna Kestrel", "Kael Dunmore", "Lira Pennywhistle",
            "Maddox Fenwright", "Nessa Holloway", "Orrin Stagg", "Petra Wynne",
            "Quill Harrow", "Rosalind Mire", "Soren Blackbriar", "Tamsin Oakes",
        ],
    },
    Table {
        title: "NPC Motivations",
        category: "NPCs",
        rows: &[
            "Repay a debt before the collector arrives",
            "Protect a younger sibling at any cost",
            "Win back the respect of their guild",
            "Hide a crime they committed years ago",
            "Find out who really owns their land",
            "Leave town without anyone noticing",
            "Prove a rival is a fraud",
            "Keep an old promise to a dying friend",
            "Get rich quickly, whatever it takes",
            "Recover a family heirloom that was stolen",
            "Earn a place among the nobility",
            "Atone for a betrayal that went wrong",
        ],
    },
    Table {
        title: "Settlements",
        category: "Locations",
        rows: &[
            "A river ford town built around a toll bridge",
            "A mining camp that struck something strange",
            "A walled market town on a busy crossroads",
            "A fishing village half sunk into the marsh",
            "A hilltop monastery town that runs on pilgrims",
            "A logging settlement at the edge of an old forest",
            "A ruined fort reoccupied by refugees",
            "A canal city of narrow bridges and quiet feuds",
            "A farming hamlet where the harvest keeps failing",
            "A border garrison town waiting for war",
        ],
    },
    Table {
        title: "Taverns and Inns",
        category: "Locations",
        rows: &[
            "The Leaning Lantern: tilted floors, honest stew",
            "The Gilded Goose: overpriced, full of gossiping merchants",
            "The Drowned Bell: sailors' haunt with a sunken bell above the bar",
            "The Last Ember: the hearth has not gone out in a hundred years",
            "The Crooked Antler: hunters' lodge with trophies on every wall",
            "The Quiet Hour: no one speaks above a whisper, by house rule",
            "The Brass Kettle: run by retired adventurers who tell tall tales",
            "The Mended Wheel: wagon drivers, card games and fistfights",
            "The Silver Moth: open only after dark, music until dawn",
            "The Hollow Oak: built inside a living tree",
        ],
    },
    Table {
        title: "Rumors",
        category: "Adventure",
        rows: &[
            "Lights move in the abandoned mill at night",
            "The mayor has not been seen in public for a month",
            "A caravan vanished on the old road with all its guards",
            "Someone is buying every map of the hills at double price",
            "The well water tastes of iron since the earthquake",
            "A stranger paid for drinks with coins from a fallen kingdom",
            "The temple bell rang by itself three nights running",
            "Wolves were seen walking upright at the forest edge",
            "The bank vault was opened, but nothing was taken",
            "A child keeps drawing a door that does not exist",
            "The lord's heir is not who everyone thinks",
            "A ship came into harbor with no crew aboard",
        ],
    },
    Table {
        title: "Adventure Hooks",
        category: "Adventure",
        rows: &[
            "A desperate parent begs the party to find a missing child",
            "A dying messenger hands over a sealed letter",
            "The party is mistaken for wanted criminals",
            "A noble hires them to guard a wedding that someone wants ruined",
            "An old friend asks for help and will not say why",
            "A map is found sewn into a secondhand coat",
            "A town offers a reward for clearing out a nearby ruin",
            "A rival group claims the party's last reward was stolen",
            "A storm strands the party somewhere they should not be",
            "A local hero vanishes the night before a festival",
        ],
    },
    Table {
        title: "Wilderness Discoveries",
        category: "Exploration",
        rows: &[
            "A circle of standing stones, warm to the touch",
            "An abandoned campsite with the fire still smoking",
            "A collapsed bridge and a hidden ford nearby",
            "A hermit's hut stocked for a long siege",
            "Giant tracks that end suddenly in a clearing",
            "A shrine to a forgotten god, recently cleaned",
            "A battlefield littered with rusted weapons",
            "A sinkhole revealing old stonework below",
            "A grove where no birds sing",
            "A merchant's wagon overturned, the cargo untouched",
            "A spring whose water glows faintly at night",
            "A watchtower with a signal fire ready to light",
        ],
    },
    Table {
        title: "Weather",
        category: "Exploration",
        rows: &[
            "Clear skies and a light breeze",
            "Thick morning fog that burns off by noon",
            "Steady rain that turns roads to mud",
            "Gusting wind that makes ranged attacks harder",
            "Oppressive heat; travelers need extra water",
            "Sudden thunderstorm with lightning strikes nearby",
            "Bitter cold and frost on every surface",
            "Snowfall that slows travel and hides tracks",
        ],
    },
    Table {
        title: "Complications",
        category: "Adventure",
        rows: &[
            "An ally changes sides at the worst moment",
            "The target is not where the information said",
            "A third party arrives wanting the same thing",
            "The path ahead collapses or floods",
            "Someone innocent is caught in the middle",
            "The reward turns out to be cursed, fake or stolen",
            "A deadline suddenly moves closer",
            "An old enemy recognizes one of the characters",
            "The local authorities step in",
            "Success reveals a much larger problem",
        ],
    },
    Table {
        title: "Treasure Ideas",
        category: "Rewards",
        rows: &[
            "A pouch of foreign coins worth more to a collector",
            "A finely made blade with a family crest",
            "A deed to a ruined property",
            "A sealed letter of credit from a distant bank",
            "A music box that plays a tune no one recognizes",
            "A gemstone with a tiny map scratched inside",
            "A favor owed by a powerful local figure",
            "A spellbook with half its pages missing",
            "A crate of rare spices worth a small fortune",
            "A key that fits no lock the party has seen yet",
            "A trained animal that chooses one character",
            "A crown that was supposed to have been destroyed",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_render_one_row_per_entry_with_a_matching_die() {
        for table in TABLES {
            assert!(table.rows.len() >= 6, "{} is too short", table.title);
            let body = table.body();
            assert!(body.contains(&format!("| {} | {} |", table.rows.len(), table.rows.last().unwrap())));
            assert!(body.starts_with(&format!("Roll a d{}", table.rows.len())));
        }
    }

    #[test]
    fn library_covers_every_020_topic() {
        assert_eq!(GUIDES.len(), 7);
        let names: Vec<_> = TABLES.iter().map(|t| t.title).collect();
        for required in ["NPC Names", "NPC Motivations", "Settlements", "Taverns and Inns", "Rumors",
            "Adventure Hooks", "Wilderness Discoveries", "Weather", "Complications", "Treasure Ideas"] {
            assert!(names.contains(&required), "missing {required}");
        }
    }
}
