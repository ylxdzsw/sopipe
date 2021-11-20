use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::runtime::RuntimeHandler;
use api::{Actor, Argument, Component};

type R = RuntimeHandler;

#[derive(pest_derive::Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

// intermediate graph presentation
struct Node {
    // TODO: record the source position of the node so we can print it out in error messages?
    comp: &'static dyn Component<R>,
    args: Vec<(String, Argument)>,
}

impl Node {
    fn build(mut self, outputs: impl IntoIterator<Item = String>) -> Box<dyn Actor<R>> {
        self.args.push(("outputs".into(), outputs.into_iter().collect()));
        self.comp.create(self.args)
    }
}

enum CNode {
    Single {
        node: Option<Node>,
        outputs: Vec<(usize, String)>,
    },
    Composite {
        forward: Option<Node>,
        backward: Option<Node>,
        output: Option<usize>,
    },
}

type CNodeIndex = usize;

enum SymbolValue {
    CNode(CNodeIndex),
    Function(&'static dyn Component<R>),
}

#[derive(Default)]
pub(crate) struct Interpreter {
    cnodes: Vec<RefCell<CNode>>,
    symbol_table: BTreeMap<String, SymbolValue>
}

impl Interpreter {
    pub(crate) fn load_script(code: &str, components: &[&'static dyn Component<R>]) -> Vec<super::Node> {
        let mut interpreter = Interpreter::default();
        for &comp in components {
            for fname in comp.functions() {
                interpreter.symbol_table.insert(fname.to_string(), SymbolValue::Function(comp));
            }
        }

        for pair in ScriptParser::parse(Rule::script, code).unwrap() {
            interpreter.eval(pair)
        }

        let Interpreter { cnodes, .. } = interpreter;
        cnodes.into_iter().map(|cnode| match cnode.into_inner() {
            CNode::Single { node, outputs } => {
                let (outputs, output_names): (Vec<_>, Vec<_>) = outputs.into_iter().unzip();
                let actor = Box::leak(node.unwrap().build(output_names));
                super::Node::new(actor, actor, outputs.leak())
            }
            CNode::Composite {
                forward,
                backward,
                output,
            } => {
                let output_names: Vec<_> = output.iter().map(|_| "".to_string()).collect();
                let forward_actor = Box::leak(forward.unwrap().build(output_names.clone()));
                let backward_actor = Box::leak(backward.unwrap().build(output_names));
                super::Node::new(
                    forward_actor,
                    backward_actor,
                    output.into_iter().collect::<Vec<_>>().leak(),
                )
            }
        }).collect()
    }

    fn eval(&mut self, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::EOI | Rule::WHITESPACE => {}
            Rule::stmt => self.eval_stmt(pair),
            Rule::script => pair.into_inner().for_each(|x| self.eval(x)),
            _ => unreachable!()
        }
    }

    fn eval_stmt(&mut self, pair: Pair<Rule>) {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::assignment => self.eval_assignment(pair),
            Rule::pipe => { self.eval_pipe(pair); }
            _ => unreachable!(),
        }
    }

    fn eval_assignment(&mut self, pair: Pair<Rule>) {
        let mut pairs = pair.into_inner();
        let ident = pairs.next().unwrap().as_str().to_string();
        let value = self.eval_pipe(pairs.next().unwrap());
        self.symbol_table.insert(ident, SymbolValue::CNode(value));
    }

    fn eval_cnode(&mut self, pair: Pair<Rule>) -> CNodeIndex {
        let mut pairs = pair.into_inner();
        let first = pairs.next().unwrap();
        if let Some(second) = pairs.next() {
            let cnode = RefCell::new(CNode::Composite { forward: None, backward: None, output: None });
            let index = self.cnodes.len();
            self.cnodes.push(cnode);

            fn make_node(this: &mut Interpreter, parent: CNodeIndex, pair: Pair<Rule>) -> Node {
                let mut pairs = pair.into_inner();
                let ident = pairs.next().unwrap().as_str();
                let mut args = pairs.next().map(|pair| this.eval_args(parent, pair)).unwrap_or_default();

                if let SymbolValue::Function(comp) = &this.symbol_table[ident] {
                    args.push(("function_name".into(), ident.to_string().into()));
                    Node { comp: *comp, args }
                } else {
                    panic!("the double bang (!!) composition can only be used to combine two function calls.")
                }
            }

            let forward_node = make_node(self, index, first);
            let backward_node = make_node(self, index, second);

            let mut cnode = self.cnodes[index].borrow_mut();
            if let CNode::Composite { forward, backward, .. } = &mut *cnode {
                *forward = Some(forward_node);
                *backward = Some(backward_node);
            } else {
                unreachable!()
            }

            index
        } else {
            let mut pairs = first.into_inner();
            let ident = pairs.next().unwrap().as_str();

            match &self.symbol_table[ident] {
                SymbolValue::Function(comp) => {
                    let comp = *comp; // to shorten the borrow of `self`
                    let cnode = RefCell::new(CNode::Single { node: None, outputs: vec![] });
                    let index = self.cnodes.len();
                    self.cnodes.push(cnode);

                    let mut args = pairs.next().map(|pair| self.eval_args(index, pair)).unwrap_or_default();
                    args.push(("function_name".into(), ident.to_string().into()));

                    let mut cnode = self.cnodes[index].borrow_mut();
                    if let CNode::Single { node, .. } = &mut *cnode {
                        *node = Some(Node { comp, args })
                    } else {
                        unreachable!()
                    }

                    index
                }
                SymbolValue::CNode(cnode) => {
                    assert!(pairs.next().is_none());
                    *cnode
                }
            }
        }
    }

    fn eval_lit(&self, pair: Pair<Rule>) -> Argument {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::string => {
                // TODO: escaping!
                let s = pair.as_str();
                Argument::String(s[1..s.len() - 1].to_string())
            }
            Rule::int => Argument::Int(pair.as_str().parse().unwrap()),
            _ => unreachable!(),
        }
    }

    fn eval_ipipe(&mut self, parent: CNodeIndex, pair: Pair<Rule>) {
        let mut pairs = pair.into_inner();
        let first = pairs.next().unwrap();
        let output_name = first.into_inner().next().map(|x| x.as_str().to_string()).unwrap_or_default();
        self._parse_pipe_rec(Some(parent), Some(output_name), pairs);
    }

    fn eval_pipe(&mut self, pair: Pair<Rule>) -> CNodeIndex {
        let mut pairs = pair.into_inner();
        if let Rule::dotted = pairs.peek().unwrap().as_rule() {
            let mut dotted_pairs = pairs.next().unwrap().into_inner();
            let first = dotted_pairs.next().unwrap();
            let second = dotted_pairs.next().unwrap();
            let cnode = if let SymbolValue::CNode(cnode) = self.symbol_table[first.as_str()] {
                cnode
            } else {
                panic!("LHS of dot expression is not a node")
            };
            let output_name = second.as_str().to_string();
            self._parse_pipe_rec(Some(cnode), Some(output_name), pairs)
        } else {
            self._parse_pipe_rec(None, None, pairs)
        }
    }

    fn _parse_pipe_rec(&mut self, last: Option<CNodeIndex>, last_output_name: Option<String>, mut rest_pairs: Pairs<Rule>) -> CNodeIndex {
        if let Some(pair) = rest_pairs.next() {
            let cnode = self.eval_cnode(pair);
            if let Some(last) = last {
                match &mut *self.cnodes[last].borrow_mut() {
                    CNode::Single { outputs, .. } => {
                        let name = last_output_name.unwrap_or_default();
                        outputs.push((cnode, name))
                    }
                    CNode::Composite { output, .. } => {
                        assert!(output.is_none());
                        *output = Some(cnode)
                    }
                }
            }
            self._parse_pipe_rec(Some(cnode), None, rest_pairs)
        } else {
            last.unwrap()
        }
    }

    fn eval_arg(&mut self, parent: CNodeIndex, pair: Pair<Rule>) -> Option<(String, Argument)> {
        let mut pairs = pair.into_inner();
        let first = pairs.next().unwrap();
        match first.as_rule() {
            Rule::lit => Some(("".to_string(), self.eval_lit(first))),
            Rule::ident => Some((
                first.as_str().to_string(),
                pairs.next().map(|x| self.eval_lit(x)).unwrap_or(Argument::None),
            )),
            Rule::ipipe => {
                self.eval_ipipe(parent, first);
                None
            }
            _ => unreachable!(),
        }
    }

    fn eval_args(&mut self, parent: CNodeIndex, pair: Pair<Rule>) -> Vec<(String, Argument)> {
        pair.into_inner()
            .flat_map(|pair| self.eval_arg(parent, pair))
            .collect()
    }
}

