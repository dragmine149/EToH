# Spec

This is an outline of what i want the front-end to be like. If you think the spec needs to be changed, feel free but just ask me first.

## Implementation
- Everything should be split up, 1 file should not contain it all.
- I want it implemented as a module so i can `mod wikitext`
- Errors should be rich explaining as much as possible

## Structs
- WikiText
- ParsedData
- Template
- Link
- List
- Text
...

## Enums
- QueryType
- LinkType
- ListType

## Outline
- WikiText
	- pub fn parse(input: &str) -> WikiText
		- must only parse when we require something inside WikiText
		- must cache so only parse once
		- everything must be owned by wikitext, let the user clone and own stuff if they need it.
	- pub fn get_parsed() -> ParsedData
		- Exposes the parsed data for manual consumption.
	- pub page_name -> Option<String>
	- pub set_page_name(page_name: Option<String>) 
	- pub text -> String
		- This is gotten from `WikiText::parse("DATA")`

- ParsedData
	- Another raw object containing everything.
	- Designed as the top-level thing that is in use all the time, you want an argument then it'll be in here.
	- pub fn get_template(name: &str) -> Result<Template>
		- Wrapper for `get_template_query` with `query_type` as exact
	- pub fn get_template_query(query: &str, query_type: QueryType) -> Vec<Template>
		- Gets all templates which matches the query_type provided.
		- Uses `self.get_parsed()`
	- pub fn get_links(type: Option<LinkType>) -> Vec<Link>
		- Returns all the top-level links of the specified type.
	- pub fn get(nth: u64) -> Argument
		- returns the nth element interacted with.
		- If it's something we don't parse, just return Argument::Text
		- For example, if a page contains 2 templates and a link and we do `.get(2)` it should return the 2nd template

- Template
	- pub arguments -> Vec<Arguments>
		- return a list of arguments.
	- pub fn get_named_arg(name: &str) -> Result<ParsedData>
		- similar to WikiText::get_template().
	- pub fn get_named_args_query(query: &str, type: QueryType) -> Vec<ParsedData>
		- similar to WikiText::get_template_query().
	- pub fn get_positional_arg(pos: &u64) -> ParsedData

- Link
	- pub type -> LinkType
	- pub label -> String
	- pub link -> String

- List
	- pub type -> ListType
	- pub entries -> Vec<Argument>

- Text
	- pub String
	- raw data of the text, also contains things we don't parse.

- QueryType
	- pub exact
	- pub startswith
	- pub contains

- LinkType
	- pub internal
	- pub external

- ListType
	- pub ordered
	- pub unordered
	- pub numbered
	- pub ... (fell free to add more if mediawiki has more)


## Notes
- Trim leading whitespaces in templates and stuff it'll just break it.

## Examples
```
{TowerInfobox
|type_of_tower=[[Tower]]
|number_of_floors=10
|found_in={{Emblem|Z6}} [[Zone 6]]
|difficulty={{DifficultyNum|4.67}}
|creator(s)={{PlayerName|treeknighterr}}
|original_difficulty={{DifficultyName|3}}
|image1=
<gallery>
ToRC image hq.png|Front
ToRC image hq backside.png|Back
</gallery>
|techniques_required=Corner Flipping
|previous_difficulty={{DifficultyNameNoLink|4|m}}
|title1={{PAGENAME}}
|caption1=ToRC as seen in Zone 6.
|date_added=6 February 2022 (confirmed)
13 May 2022 (released)}}
```
I want to be able to do the following:
```rs
let wt = WikiText::parse("<example>");
let tower = wt.get_template("towerinfobox", QueryType::Exact);
```

```rs
let diff = tower
		.get_named_arg("difficulty")?
		.get_template("difficultynum")?
		.get_positional_argument(0)?
		.prase::<f64>()
		.unwrap_or(100.0);
```
```rs
let location = tower
		.get_named_arg("found_in", QueryType::Startswith)
		.get_template("emblem")?
		.get_positional_argument(0)?;
```
