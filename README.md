# SimpleDOT Language Parser

## Goal

Write a parser for a subset (which we'll call SimpleDOT) of the GraphViz [DOT Language](https://graphviz.org/doc/info/lang.html). The intention to parse input of this SimpleDOT language into an in-memory structure for visualization purposes.  This will open the way for additional libraries to actually perform the visualization.

## Status

Work in progress. Currently most work is going into the initial step: parsing the data into
an intermediate representation that can then be parsed into the actual syntax tree.

## DOT Language Grammar

The original DOT Language is [defined](https://graphviz.org/doc/info/lang.html) as

| Name         | Rule                                                                                                                             |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| _graph_      | [ **strict** ] (**graph** &#124; **digraph**) [ _ID_ ] **'{'** _stmt_list_ **'}'**                                               |
| _stmt_list_  | [ _stmt_ [ **';'** ] _stmt_list_ ]                                                                                               |
| _stmt_       | _node_stmt_                                                                                                                      |
|              | _edge_stmt_                                                                                                                      |
|              | _attr_stmt_                                                                                                                      |
|              | _ID_ **'='** _ID_                                                                                                                |
|              | _subgraph_                                                                                                                       |
| _attr_stmt_  | (**graph** &#124; **node** &#124; **edge**) _attr_list_                                                                          |
| _attr_list_  | **'['** [ _a_list_ ] **']'** [ _attr_list_ ]                                                                                     |
| _a_list_     | _ID_ **'='** _ID_ [ (**';'** &#124; **','**) ] [ _a_list_ ]                                                                      |
| _edge_stmt_  | (_node_id_ &#124; _subgraph_) _edge_rhs_ [ _attr_list_ ]                                                                         |
| _edge_rhs_   | _edgeop_ (_node_id_ &#124; _subgraph_) [ _edge_rhs_ ]                                                                            |
| _node_stmt_  | _node_id_ [ _attr_list_ ]                                                                                                        |
| _node_id_    | _ID_ [ _port_ ]                                                                                                                  |
| _port_       | **':'** _ID_ [ **':'** _compass_pt_ ]                                                                                            |
|              | **':'** _compass_pt_                                                                                                             |
| _subgraph_   | [ **subgraph** [ _ID_ ] ] **'{'** _stmt_list_ **'}'**                                                                            |
| _compass_pt_ | (**n** &#124; **ne** &#124; **e** &#124; **se** &#124; **s** &#124; **sw** &#124; **w** &#124; **nw** &#124; **c** &#124; **_**) |


Where _ID_ is one of the following:
-   Any string of alphabetic (`[a-zA-Z\200-\377]`) characters, underscores (`'_'`) or digits(`[0-9]`), not beginning with a digit;
-   a numeral [`-`]?(`.`[`0`-`9`]⁺ `|` [`0`-`9`]⁺(`.`[`0`-`9`]*)? );
-   any double-quoted string (`"..."`) possibly containing escaped quotes (`\"`)¹;
-   an HTML string (`<...>`).

## SimpleDOT Language Grammar

The SimpleDOT language is a small subset of the full DOT language intended to limit support to what is needed to provide descriptions of fairly simple tree structures. Any graph definition defined in SimpleDOT should be parseable and renderable by standard DOT parsers and renderers.

| Name        | Rule                                                                               |
| ----------- | ---------------------------------------------------------------------------------- |
| _graph_     | [ **strict** ] (**graph** &#124; **digraph**) [ _ID_ ] **'{'** _stmt_list_ **'}'** |
| _stmt_list_ | [ _stmt_ [ **';'** ] _stmt_list_ ]                                                 |
| _stmt_      | _node_stmt_                                                                        |
|             | _edge_stmt_                                                                        |
|             | _attr_stmt_                                                                        |
|             | _ID_ **'='** _ID_                                                                  |
|             | _subgraph_                                                                         |
| _attr_stmt_ | (**graph** &#124; **node** &#124; **edge**) _attr_list_                            |
| _attr_list_ | **'['** [ _a_list_ ] **']'** [ _attr_list_ ]                                       |
| _a_list_    | _ID_ **'='** _ID_ [ (**';'** &#124; **','**) ] [ _a_list_ ]                        |
| _edge_stmt_ | _node_id_ _edge_rhs_ [ _attr_list_ ]                                               |
| _edge_rhs_  | _edgeop_ _node_id_ [ _edge_rhs_ ]                                                  |
| _node_stmt_ | _node_id_ [ _attr_list_ ]                                                          |
| _node_id_   | _ID_                                                                               |

Where _ID_ is the same as in the full DOT language with the exception of HTML strings, which are excluded for simplicity reasons.

Otherwise, at a purely language grammar level, the only real change is the removal of the _subgraph_, _port_, and _compass_pt_ constructs.

### SimpleDOT Supported Attributes

A more significant change between the full DOT langauge and SimpleDOT is a significantly limited attribute set, as described by the following table.  The **Used By** colum uses the characters `E`, `N`, and `G` to denote whether the attribute applies to edge, node, or graph, respectively.  The **Type** column refers to [attribute types](https://graphviz.org/docs/attr-types/) in full DOT language definition.

| Name         | Used By | Type                | Default                          |
| ------------ | ------- | ------------------- | -------------------------------- |
| `bgcolor`    | G       | `color`,`colorList` | `<none>`                         |
| `color`      | EN      | `color`,`colorList` | `black`                          |
| `comment`    | ENG     | `string`            | `""`                             |
| `fontcolor`  | G       | `color`             | `black`                          |
| `fontname`   | G       | `string`            | `Times-Roman`                    |
| `fontsize`   | G       | `double`            | `14.0`                           |
| `height`     | N       | `double`            | `0.5`                            |
| `image`      | N       | `string`            | `""`                             |
| `imagepos`   | N       | `string`            | `""`                             |
| `imagescale` | N       | `bool`,`string`     | `false`                          |
| `label`      | ENG     | `lblString`         | `"\N"` (nodes), `""` (otherwise) |
| `width`      | N       | `double`            | `0.75`                           |

## Approach

We're going to use the [`nom`](https://github.com/Geal/nom) parser combinators library to construct the language parser. 

## Target Data Structure

Rough draft:

```rust
enum GraphKind {
	Directed,
	Undirected
}

struct Graph {
	kind: GraphKind,
	strict: boolean,
	name: String,
	attributes: Vec<GraphAttribute>,
	component: Vec<Component>
}

enum Component {
	Edge(Edge),
	Node(Node),
	Subgraph(Graph),
	ClusterSubgraph(Graph)
}

struct GraphAttribute {
	name: AttributeName,
	value: String,
}
```
