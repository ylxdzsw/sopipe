use std::collections::BTreeMap;
use std::cell::RefCell;
use pest::Parser;
use pest::iterators::Pair;

use api::*;

#[derive(pest_derive::Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

// intermediate graph presentation
struct Node {
    // TODO: record the source position of the node so we can print it out in error messages?
    comp: &'static dyn Component,
    args: Vec<(String, Argument)>,
}

impl Node {
    fn build(mut self, outputs: impl IntoIterator<Item=String>) -> api::Result<Box<dyn api::Actor>> {
        self.args.push(("outputs".into(), outputs.into_iter().collect()));
        self.comp.create(self.args)
    }
}

enum CNode {
    Single { node: Node, outputs: Vec<usize> },
    Composite { forward: Node, backward: Node, output: Option<usize> },
}

type CNodeIndex = usize;

/// load a script, build the DAG, initialize the nodes:
pub(crate) fn load_script(code: &str, components: &[&'static dyn Component]) -> Result<Vec<super::Node>> {
    enum SymbolValue { CNode(CNodeIndex), Function(&'static dyn Component) }

    let mut cnodes = vec![];

    let mut symbol_table: BTreeMap<String, SymbolValue> = BTreeMap::new();
    for &comp in components {
        for fname in comp.functions() {
            symbol_table.insert(fname.to_string(), SymbolValue::Function(comp));
        }
    }

    fn get_lit_value(pair: Pair<Rule>) -> Argument {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::string => Argument::String(pair.as_str().to_string()), // TODO: escaping!
            Rule::int => Argument::Int(pair.as_str().parse().unwrap()),
            _ => unreachable!()
        }
    }

    fn parse_arg(pair: Pair<Rule>) -> (String, Argument) {
        let mut pairs = pair.into_inner();
        let first = pairs.next().unwrap();
        match first.as_rule() {
            Rule::lit => ("".to_string(), get_lit_value(first)),
            Rule::ident => (first.as_str().to_string(), pairs.next().map(get_lit_value).unwrap_or(Argument::None)),
            _ => unreachable!()
        }
    }

    fn parse_node(pair: Pair<Rule>) -> (String, Vec<(String, Argument)>) {
        assert_eq!(pair.as_rule(), Rule::node);
        let mut pairs = pair.into_inner();
        let ident = pairs.next().unwrap().as_str().to_string();
        let args = pairs.next().map(|x| x.into_inner().map(parse_arg).collect()).unwrap_or_default();
        (ident, args)
    }

    fn eval(symbol_table: &mut BTreeMap<String, SymbolValue>, cnodes: &mut Vec<RefCell<CNode>>, pair: Pair<Rule>) -> CNodeIndex {
        match pair.as_rule() {
            Rule::cnode => {
                let mut pairs = pair.into_inner();
                let first = pairs.next().unwrap();
                if let Some(second) = pairs.next() {
                    fn make_node(pair: Pair<Rule>, symbol_table: &mut BTreeMap<String, SymbolValue>) -> Node {
                        let (ident, mut args) = parse_node(pair);
                        if let SymbolValue::Function(comp) = &symbol_table[&ident] {
                            args.push(("function_name".into(), ident.into()));
                            Node { comp: *comp, args }
                        } else {
                            panic!("the double bang (!!) composition can only be used to combine two function calls.")
                        }
                    }
                    let cnode = RefCell::new(CNode::Composite {
                        forward: make_node(first, symbol_table),
                        backward: make_node(second, symbol_table),
                        output: None
                    });
                    let index = cnodes.len();
                    cnodes.push(cnode);
                    index
                } else {
                    let (ident, mut args) = parse_node(first);
                    match &symbol_table[&ident] {
                        SymbolValue::Function(comp) => {
                            args.push(("function_name".into(), ident.into()));
                            let index = cnodes.len();
                            let cnode = RefCell::new(CNode::Single {
                                node: Node { comp: *comp, args },
                                outputs: vec![]
                            });
                            index
                        }
                        SymbolValue::CNode(cnode) => {
                            cnode.clone()
                        },
                    }
                }
            },
            Rule::pipe => {
                let mut last: Option<CNodeIndex> = None;
                for pair in pair.into_inner() {
                    let cnode_index = eval(symbol_table, cnodes, pair);
                    if let Some(p) = last {
                        match &mut *cnodes[p].borrow_mut() {
                            CNode::Single { outputs, .. } => outputs.push(cnode_index),
                            CNode::Composite { output, .. } => {
                                assert!(output.is_none());
                                *output = Some(cnode_index)
                            }
                        }
                    }
                    last = Some(cnode_index)
                }
                last.unwrap()
            },
            _ => unreachable!(),
        }
    }

    fn walk(symbol_table: &mut BTreeMap<String, SymbolValue>, cnodes: &mut Vec<RefCell<CNode>>, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::EOI | Rule::WHITESPACE => {},
            Rule::stmt => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::assignment => walk(symbol_table, cnodes, pair),
                    Rule::pipe => { eval(symbol_table, cnodes, pair); },
                    _ => unreachable!()
                }
            },
            Rule::assignment => {
                let mut pairs = pair.into_inner();
                let ident = pairs.next().unwrap().as_str().to_string();
                let value = eval(symbol_table, cnodes, pairs.next().unwrap());
                symbol_table.insert(ident, SymbolValue::CNode(value));
            },
            Rule::script => for pair in pair.into_inner() {
                walk(symbol_table, cnodes, pair)
            },
            _ => unreachable!()
        }
    }

    for pair in ScriptParser::parse(Rule::script, code)? {
        walk(&mut symbol_table, &mut cnodes, pair)
    }

    cnodes.into_iter().map(|cnode| {
        match cnode.into_inner() {
            CNode::Single { node, outputs } => {
                let output_names: Vec<_> = outputs.iter().map(|_| "".to_string()).collect();
                let actor = Box::leak(node.build(output_names)?);
                Ok(super::Node::new(actor, actor, outputs.leak()))
            }
            CNode::Composite { forward, backward, output } => {
                let output_names: Vec<_> = output.iter().map(|_| "".to_string()).collect();
                let forward_actor = Box::leak(forward.build(output_names.clone())?);
                let backward_actor = Box::leak(backward.build(output_names)?);
                Ok(super::Node::new(forward_actor, backward_actor, output.into_iter().collect::<Vec<_>>().leak()))
            }
        }
    }).collect()
}
