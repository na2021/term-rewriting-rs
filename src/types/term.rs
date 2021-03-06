use super::super::pretty::Pretty;
use super::{Atom, Operator, Place, Unification, Variable};
use itertools::Itertools;
use std::collections::HashMap;
use std::iter;

/// A first-order `Context`: a [`Term`] that may have [`Hole`]s; a sort of [`Term`] template.
///
/// [`Term`]: enum.Term.html
/// [`Hole`]: enum.Context.html#variant.Hole
///
/// Examples
///
/// ```
/// # use term_rewriting::{Signature, Context, parse_context};
/// let mut sig = Signature::default();
///
/// // Constructing a Context manually.
/// let a = sig.new_op(3, Some("A".to_string()));
/// let b = sig.new_op(0, Some("B".to_string()));
/// let x = sig.new_var(Some("x".to_string()));
///
/// let b_context = Context::Application { op: b, args: vec![] };
/// let x_context = Context::Variable(x);
///
/// let context = Context::Application { op: a, args: vec![ b_context, x_context, Context::Hole ] };
///
/// // Constructing a Context using the Parser.
/// let context2 = parse_context(&mut sig, "A(B x_ [!])").expect("parse of A(B x_ [!])");
///
/// assert_eq!(context.display(), context2.display());
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Context {
    /// An empty place in the `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Context};
    /// // Constructing a hole manually.
    /// let h = Context::Hole;
    ///
    /// // Constructing a hole using the parser.
    /// let mut sig = Signature::default();
    /// let h2 = parse_context(&mut sig, "[!]").expect("parse of [!]");
    ///
    /// assert_eq!(h.display(), h2.display());
    /// ```
    Hole,
    /// A concrete but unspecified `Context` (e.g. `x`, `y`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Context};
    /// let mut sig = Signature::default();
    ///
    /// // Constructing a Context Variable manually.
    /// let v = sig.new_var(Some("x".to_string()));
    /// let var = Context::Variable(v);
    ///
    /// //Contstructing a Context Variable using the parser.
    /// let var2 = parse_context(&mut sig, "x_").expect("parse of x_");
    ///
    /// assert_eq!(var.display(), var2.display());
    /// ```
    Variable(Variable),
    /// An [`Operator`] applied to zero or more `Context`s (e.g. (`f(x, y)`, `g()`)
    ///
    /// [`Operator`]: struct.Operator.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Context};
    /// let mut sig = Signature::default();
    ///
    /// // Constructing a Context Application manually.
    /// let a = sig.new_op(0, Some("A".to_string()));
    /// let app = Context::Application { op: a, args: vec![] };
    ///
    /// // Constructing a Context Application using the parser.
    /// let app2 = parse_context(&mut sig, "A").expect("parse of A");
    ///
    /// assert_eq!(app, app2);
    /// ```
    Application { op: Operator, args: Vec<Context> },
}
impl Context {
    /// Serialize a `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, Context, Variable, Operator, parse_context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "x_ [!] A CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)")
    ///     .expect("parse of x_ [!] A CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)") ;
    ///
    /// assert_eq!(context.display(), ".(.(.(.(x_ [!]) A) CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL)))) DECC(DECC(DIGIT(1) 0) 5))");
    /// ```
    pub fn display(&self) -> String {
        match self {
            Context::Hole => "[!]".to_string(),
            Context::Variable(v) => v.display(),
            Context::Application { op, args } => {
                let op_str = op.display();
                if args.is_empty() {
                    op_str
                } else {
                    let args_str = args.iter().map(|arg| arg.display()).join(" ");
                    format!("{}({})", op_str, args_str)
                }
            }
        }
    }
    /// A human-readable serialization of the `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "x_ [!] A CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)")
    ///     .expect("parse of x_ [!] A CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)") ;
    ///
    /// assert_eq!(context.pretty(), "x_ [!] A [2, 1, 0] 105");
    /// ```
    pub fn pretty(&self) -> String {
        Pretty::pretty(self)
    }
    /// Every [`Atom`] used in the `Context`.
    ///
    /// [`Atom`]: enum.Atom.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Context, parse_context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A(B x_ [!])").expect("parse of A(B x_ [!])");
    ///
    /// let atoms: Vec<String> = context.atoms().iter().map(|a| a.display()).collect();
    ///
    /// assert_eq!(atoms, vec!["x_", "B", "A"]);
    /// ```
    pub fn atoms(&self) -> Vec<Atom> {
        let vars = self.variables().into_iter().map(Atom::Variable);
        let ops = self.operators().into_iter().map(Atom::Operator);
        vars.chain(ops).collect()
    }
    /// Every [`Variable`] used in the `Context`.
    ///
    /// [`Variable`]: struct.Variable.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A([!]) B y_ z_").expect("parse of A([!]) B y_ z_");
    ///
    /// let var_names: Vec<String> = context.variables().iter().map(|v| v.display()).collect();
    ///
    /// assert_eq!(var_names, vec!["y_".to_string(), "z_".to_string()]);
    /// ```
    pub fn variables(&self) -> Vec<Variable> {
        match *self {
            Context::Hole => vec![],
            Context::Variable(ref v) => vec![v.clone()],
            Context::Application { ref args, .. } => {
                args.iter().flat_map(Context::variables).unique().collect()
            }
        }
    }
    /// Every [`Operator`] used in the `Context`.
    ///
    /// [`Operator`]: struct.Operator.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A([!]) B y_ z_").expect("parse of A([!]) B y_ z_");
    ///
    /// let op_names: Vec<String> = context.operators().iter().map(|v| v.display()).collect();
    ///
    /// assert_eq!(op_names, vec!["A".to_string(), "B".to_string(), ".".to_string()]);
    /// ```
    pub fn operators(&self) -> Vec<Operator> {
        if let Context::Application { ref op, ref args } = *self {
            args.iter()
                .flat_map(Context::operators)
                .chain(iter::once(op.clone()))
                .unique()
                .collect()
        } else {
            vec![]
        }
    }
    /// A list of the [`Place`]s in the `Context` that are `Hole`s.
    ///
    /// [`Place`]: type.Place.html
    /// [`Hole`]: enum.Context.html#variant.Hole
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Place};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A([!] B([!]) y_ z_)").expect("parse of A([!] B([!]) y_ z_)");
    ///
    /// let p: &[usize] = &[0];
    /// let p2: &[usize] = &[1, 0];
    ///
    /// assert_eq!(context.holes(), vec![p, p2]);
    /// ```
    pub fn holes(&self) -> Vec<Place> {
        self.subcontexts()
            .into_iter()
            .filter_map(|(c, p)| {
                if let Context::Hole = *c {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    }
    /// The head of the `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Context, parse_context, Atom};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A(B([!]) z_)").expect("parse of A(B([!]) z_)");
    ///
    /// assert_eq!(context.head().unwrap().display(), "A");
    /// ```
    pub fn head(&self) -> Option<Atom> {
        match self {
            Context::Hole => None,
            Context::Variable(v) => Some(Atom::Variable(v.clone())),
            Context::Application { op, .. } => Some(Atom::Operator(op.clone())),
        }
    }
    /// The args of the `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Context, parse_context, Atom};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A B").expect("parse of A B");
    /// let args: Vec<String> = context.args().iter().map(|arg| arg.display()).collect();
    ///
    /// assert_eq!(args, vec!["A", "B"]);
    ///
    /// let context = parse_context(&mut sig, "A(y_)").expect("parse of A(y_)");
    /// let args: Vec<String> = context.args().iter().map(|arg| arg.display()).collect();
    ///
    /// assert_eq!(args, vec!["y_"]);
    /// ```
    pub fn args(&self) -> Vec<Context> {
        if let Context::Application { args, .. } = self {
            args.clone()
        } else {
            vec![]
        }
    }
    /// Every `subcontext` and its [`Place`], starting with the original `Context` itself.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context, Context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A(B [!])").expect("parse of A(B [!])");
    ///
    /// let p: Vec<usize> = vec![];
    /// let subcontext0 = parse_context(&mut sig, "A(B [!])").expect("parse of A(B [!])");
    ///
    /// let p1: Vec<usize> = vec![0];
    /// let subcontext1 = parse_context(&mut sig, "B").expect("parse of B");
    ///
    /// let p2: Vec<usize> = vec![1];
    /// let subcontext2 = Context::Hole;
    ///
    /// assert_eq!(context.subcontexts(), vec![(&subcontext0, p), (&subcontext1, p1), (&subcontext2, p2)]);
    /// ```
    pub fn subcontexts(&self) -> Vec<(&Context, Place)> {
        if let Context::Application { ref args, .. } = *self {
            let here = iter::once((self, vec![]));
            let subcontexts = args.iter().enumerate().flat_map(|(i, arg)| {
                arg.subcontexts()
                    .into_iter()
                    .zip(iter::repeat(i))
                    .map(|((t, p), i)| {
                        let mut a = vec![i];
                        a.extend(p);
                        (t, a)
                    })
            });
            here.chain(subcontexts).collect()
        } else {
            vec![(self, vec![])]
        }
    }
    /// The number of distinct [`Place`]s in the `Context`.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Context, parse_context};
    /// let mut sig = Signature::default();
    /// let context = parse_context(&mut sig, "A B").expect("parse of A B");
    ///
    /// assert_eq!(context.size(), 3);
    ///
    /// let context = parse_context(&mut sig, "A(B)").expect("parse of A(B)");
    ///
    /// assert_eq!(context.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        self.subcontexts().len()
    }
    /// Get the `subcontext` at the given [`Place`], or `None` if the [`Place`] does not exist.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Context, parse_context};
    /// let mut sig = Signature::default();
    /// let context = parse_context(&mut sig, "B(A)").expect("parse of B(A)");
    ///
    /// let p: &[usize] = &[7];
    ///
    /// assert_eq!(context.at(p), None);
    ///
    /// let p: &[usize] = &[0];
    ///
    /// assert_eq!(context.at(p).unwrap().display(), "A");
    /// ```
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::ptr_arg))]
    pub fn at(&self, place: &[usize]) -> Option<&Context> {
        self.at_helper(&*place)
    }
    fn at_helper(&self, place: &[usize]) -> Option<&Context> {
        if place.is_empty() {
            return Some(self);
        }
        match *self {
            Context::Application { ref args, .. } if place[0] <= args.len() => {
                args[place[0]].at_helper(&place[1..].to_vec())
            }
            _ => None,
        }
    }
    /// Create a copy of the `Context` where the subcontext at the given [`Place`] has been replaced with
    /// `subcontext`.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Context, parse_context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "B(A)").expect("parse of B(A)");
    /// let context2 = parse_context(&mut sig, "C [!]").expect("parse of C [!]");
    ///
    /// let p: &[usize] = &[0];
    /// let new_context = context.replace(p, context2);
    ///
    /// assert_eq!(new_context.unwrap().pretty(), "B(C [!])");
    /// ```
    pub fn replace(&self, place: &[usize], subcontext: Context) -> Option<Context> {
        self.replace_helper(&*place, subcontext)
    }
    fn replace_helper(&self, place: &[usize], subcontext: Context) -> Option<Context> {
        if place.is_empty() {
            Some(subcontext)
        } else {
            match *self {
                Context::Application { ref op, ref args } if place[0] <= args.len() => {
                    if let Some(context) =
                        args[place[0]].replace_helper(&place[1..].to_vec(), subcontext)
                    {
                        let mut new_args = args.clone();
                        new_args.remove(place[0]);
                        new_args.insert(place[0], context);
                        Some(Context::Application {
                            op: op.clone(),
                            args: new_args,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
    /// Translate the `Context` into a [`Term`], if possible.
    ///
    /// [`Term`]: enum.Term.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_context};
    /// let mut sig = Signature::default();
    ///
    /// let context = parse_context(&mut sig, "A(B [!])").expect("parse of A(B [!])");
    ///
    /// assert!(context.to_term().is_err());
    ///
    /// let context = parse_context(&mut sig, "A(B C)").expect("parse of A(B C)");
    ///
    /// let term = context.to_term().expect("converting context to term");
    ///
    /// assert_eq!(term.display(), "A(B C)");
    /// ```
    pub fn to_term(&self) -> Result<Term, ()> {
        match *self {
            Context::Hole => Err(()),
            Context::Variable(ref v) => Ok(Term::Variable(v.clone())),
            Context::Application { ref op, ref args } => {
                let mut mapped_args = vec![];
                for arg in args {
                    mapped_args.push(arg.to_term()?);
                }
                Ok(Term::Application {
                    op: op.clone(),
                    args: mapped_args,
                })
            }
        }
    }
}
impl From<Term> for Context {
    fn from(t: Term) -> Context {
        match t {
            Term::Variable(v) => Context::Variable(v),
            Term::Application { op, args } => {
                let args = args.into_iter().map(Context::from).collect();
                Context::Application { op, args }
            }
        }
    }
}

/// A first-order term: either a [`Variable`] or an application of an [`Operator`].
///
/// [`Variable`]: struct.Variable.html
/// [`Operator`]: struct.Operator.html
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Term {
    /// A concrete but unspecified `Term` (e.g. `x`, `y`).
    /// See [`Variable`] for more information.
    ///
    /// [`Variable`]: struct.Variable.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// // Constructing a Variable manually
    /// let var = sig.new_var(Some("x_".to_string()));
    /// let var_term = Term::Variable(var);
    ///
    /// // Constructing a Variable using the parser
    /// let var = parse_term(&mut sig, "x_");
    /// ```
    Variable(Variable),
    /// An [`Operator`] applied to zero or more `Term`s (e.g. (`f(x, y)`, `g()`).
    ///
    /// A `Term` that is an application of an [`Operator`] with arity 0 applied to 0 `Term`s can be considered a constant.
    ///
    /// [`Operator`]: struct.Operator.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// // Constructing a Constant manually
    /// let a = sig.new_op(0, Some("A".to_string()));
    /// let const_term = Term::Application {
    ///     op: a,
    ///      args: vec![],
    /// };
    ///
    /// // Constructing a Constant using the parser
    /// let const_term = parse_term(&mut sig, "A");
    ///
    /// // Constructing an Application manually
    /// let x = sig.new_var(Some("x_".to_string()));
    /// let b = sig.new_op(1, Some("B".to_string()));
    /// let op_term = Term::Application {
    ///     op: b,
    ///     args: vec![Term::Variable(x)],
    /// };
    ///
    /// // Constructing an Application using the parser
    /// let op_term = parse_term(&mut sig, "B(x_)");
    /// ```
    Application { op: Operator, args: Vec<Term> },
}
impl Term {
    /// Serialize a `Term`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// let term = parse_term(&mut sig, "A B(x_) CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)")
    ///     .expect("parse of A B(x_) CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)");
    ///
    /// assert_eq!(term.display(), ".(.(.(A B(x_)) CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL)))) DECC(DECC(DIGIT(1) 0) 5))");
    /// ```
    pub fn display(&self) -> String {
        match self {
            Term::Variable(ref v) => v.display(),
            Term::Application { ref op, ref args } => {
                let op_str = op.display();
                if args.is_empty() {
                    op_str
                } else {
                    let args_str = args.iter().map(|arg| arg.display()).join(" ");
                    format!("{}({})", op_str, args_str)
                }
            }
        }
    }
    /// A human-readable serialization of the `Term`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// let term = parse_term(&mut sig, "A B(x_) CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)")
    ///     .expect("parse of A B(x_) CONS(SUCC(SUCC(ZERO)) CONS(SUCC(ZERO) CONS(ZERO NIL))) DECC(DECC(DIGIT(1) 0) 5)");
    ///
    /// assert_eq!(term.pretty(), "A B(x_) [2, 1, 0] 105");
    /// ```
    pub fn pretty(&self) -> String {
        Pretty::pretty(self)
    }
    /// Every [`Atom`] used in the `Term`.
    ///
    /// [`Atom`]: enum.Atom.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// let example_term = parse_term(&mut sig, "A(B x_)").expect("parse of A(B x_)");
    /// let atoms: Vec<String> = example_term.atoms().iter().map(|a| a.display()).collect();
    ///
    /// assert_eq!(atoms, vec!["x_", "B", "A"]);
    /// ```
    pub fn atoms(&self) -> Vec<Atom> {
        let vars = self.variables().into_iter().map(Atom::Variable);
        let ops = self.operators().into_iter().map(Atom::Operator);
        vars.chain(ops).collect()
    }
    /// Every [`Variable`] used in the `Term`.
    ///
    /// [`Variable`]: struct.Variable.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term, Term};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "A B y_ z_").expect("parse of A B y_ z_");
    /// let var_names: Vec<String> = t.variables().iter().map(|v| v.display()).collect();
    ///
    /// assert_eq!(var_names, vec!["y_", "z_"]);
    /// ```
    pub fn variables(&self) -> Vec<Variable> {
        match *self {
            Term::Variable(ref v) => vec![v.clone()],
            Term::Application { ref args, .. } => {
                args.iter().flat_map(Term::variables).unique().collect()
            }
        }
    }
    /// Every [`Operator`] used in the `Term`.
    ///
    /// [`Operator`]: struct.Operator.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term, Term};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "A B y_ z_").expect("parse of A B y_ z_");
    /// let op_names: Vec<String> = t.operators().iter().map(|v| v.display()).collect();
    ///
    /// assert_eq!(op_names, vec!["A", "B", "."]);
    /// ```
    pub fn operators(&self) -> Vec<Operator> {
        match *self {
            Term::Variable(_) => vec![],
            Term::Application { ref op, ref args } => args
                .iter()
                .flat_map(Term::operators)
                .chain(iter::once(op.clone()))
                .unique()
                .collect(),
        }
    }
    /// The head of the `Term`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term, Atom};
    /// let mut sig = Signature::default();
    ///
    /// let op = sig.new_op(2, Some("A".to_string()));
    /// let t = parse_term(&mut sig, "A(B z_)").expect("parse of A(B z_)");
    ///
    /// assert_eq!(t.atoms().len(), 3);
    /// assert_eq!(t.head(), Atom::Operator(op));
    /// ```
    pub fn head(&self) -> Atom {
        match self {
            Term::Variable(v) => Atom::Variable(v.clone()),
            Term::Application { op, .. } => Atom::Operator(op.clone()),
        }
    }
    /// The arguments of the `Term`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term, Atom};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "C(A B)").expect("parse of C(A B)");
    /// let arg0 = parse_term(&mut sig, "A").expect("parse of A");
    /// let arg1 = parse_term(&mut sig, "B").expect("parse of B");
    ///
    /// assert_eq!(t.args(), vec![arg0, arg1]);
    ///
    /// let t2 = parse_term(&mut sig, "A").expect("parse of A");
    ///
    /// assert_eq!(t2.args(), vec![]);
    /// ```
    pub fn args(&self) -> Vec<Term> {
        match self {
            Term::Variable(_) => vec![],
            Term::Application { args, .. } => args.clone(),
        }
    }
    /// Every `subterm` and its [`Place`], starting with the `Term` and the empty [`Place`].
    ///
    /// [`Place`]: struct.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term, Term};
    /// let mut sig = Signature::default();
    ///
    /// let b = sig.new_op(0, Some("B".to_string()));
    /// let a = sig.new_op(1, Some("A".to_string()));
    ///
    /// let p: Vec<usize> = vec![];
    /// let p1: Vec<usize> = vec![0];
    /// let t = parse_term(&mut sig, "A(B)").expect("parse of A(B)");
    /// let subterm0 = Term::Application {
    ///     op: a.clone(),
    ///     args: vec![Term::Application {
    ///         op: b.clone(),
    ///         args: vec![],
    ///     }],
    /// };
    /// let subterm1 = Term::Application {
    ///     op: b.clone(),
    ///     args: vec![],
    /// };
    ///
    /// assert_eq!(t.subterms(), vec![(&subterm0, p), (&subterm1, p1)]);
    /// ```
    pub fn subterms(&self) -> Vec<(&Term, Place)> {
        match *self {
            Term::Variable(_) => vec![(self, vec![])],
            Term::Application { ref args, .. } => {
                let here = iter::once((self, vec![]));
                let subterms = args.iter().enumerate().flat_map(|(i, arg)| {
                    arg.subterms()
                        .into_iter()
                        .zip(iter::repeat(i))
                        .map(|((t, p), i)| {
                            let mut a = vec![i];
                            a.extend(p);
                            (t, a)
                        })
                });
                here.chain(subterms).collect()
            }
        }
    }
    /// The number of distinct [`Place`]s in the `Term`.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "A B").expect("parse of A B");
    ///
    /// assert_eq!(t.size(), 3);
    ///
    /// let t = parse_term(&mut sig, "A(B)").expect("parse of A(B)");
    ///
    /// assert_eq!(t.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        self.subterms().len()
    }
    /// Get the `subterm` at the given [`Place`] if possible.  Otherwise, return `None`.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    /// let op = sig.new_op(0, Some("A".to_string()));
    /// let t = parse_term(&mut sig, "B(A)").expect("parse of B(A)");
    ///
    /// assert_eq!(t.size(), 2);
    /// let p: &[usize] = &[7];
    ///
    /// assert_eq!(t.at(p), None);
    ///
    /// let p: &[usize] = &[0];
    /// let args = vec![];
    ///
    /// assert_eq!(t.at(p), Some(&Term::Application { op, args }));
    /// ```
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::ptr_arg))]
    pub fn at(&self, place: &[usize]) -> Option<&Term> {
        self.at_helper(&*place)
    }
    fn at_helper(&self, place: &[usize]) -> Option<&Term> {
        if place.is_empty() {
            Some(self)
        } else {
            match *self {
                Term::Variable(_) => None,
                Term::Application { ref args, .. } => {
                    if place[0] <= args.len() {
                        args[place[0]].at_helper(&place[1..].to_vec())
                    } else {
                        None
                    }
                }
            }
        }
    }
    /// Create a copy of the `Term` where the `Term` at the given [`Place`] has been replaced with
    /// `subterm`.
    ///
    /// [`Place`]: type.Place.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "B(A)").expect("parse of B(A)");
    /// let t2 = parse_term(&mut sig, "C").expect("parse of C");
    /// let expected_term = parse_term(&mut sig, "B(C)").expect("parse of B(C)");
    ///
    /// let p: &[usize] = &[0];
    /// let new_term = t.replace(p, t2);
    ///
    /// assert_eq!(new_term, Some(expected_term));
    /// ```
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::ptr_arg))]
    pub fn replace(&self, place: &[usize], subterm: Term) -> Option<Term> {
        self.replace_helper(&*place, subterm)
    }
    fn replace_helper(&self, place: &[usize], subterm: Term) -> Option<Term> {
        if place.is_empty() {
            Some(subterm)
        } else {
            match *self {
                Term::Application { ref op, ref args } if place[0] <= args.len() => {
                    if let Some(term) = args[place[0]].replace_helper(&place[1..].to_vec(), subterm)
                    {
                        let mut new_args = args.clone();
                        new_args.remove(place[0]);
                        new_args.insert(place[0], term);
                        Some(Term::Application {
                            op: op.clone(),
                            args: new_args,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
    /// Given a mapping from [`Variable`]s to `Term`s, perform a substitution.
    ///
    /// [`Variable`]: struct.Variable.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term, Term};
    /// # use std::collections::HashMap;
    /// let mut sig = Signature::default();
    ///
    /// let term_before = parse_term(&mut sig, "S K y_ z_").expect("parse of S K y_ z_");
    /// let s_term = parse_term(&mut sig, "S").expect("parse of S");
    /// let k_term = parse_term(&mut sig, "K").expect("parse of K");
    ///
    /// let vars = sig.variables();
    /// let y = &vars[0];
    /// let z = &vars[1];
    ///
    /// let mut sub = HashMap::new();
    /// sub.insert(y.clone(), s_term);
    /// sub.insert(z.clone(), k_term);
    ///
    /// let expected_term = parse_term(&mut sig, "S K S K").expect("parse of S K S K");
    /// let subbed_term = term_before.substitute(&sub);
    ///
    /// assert_eq!(subbed_term, expected_term);
    /// ```
    pub fn substitute(&self, sub: &HashMap<Variable, Term>) -> Term {
        match *self {
            Term::Variable(ref v) => sub.get(v).unwrap_or(self).clone(),
            Term::Application { ref op, ref args } => Term::Application {
                op: op.clone(),
                args: args.iter().map(|t| t.substitute(sub)).collect(),
            },
        }
    }
    /// Take a vector of pairs of terms and perform a substitution on each term.
    fn constraint_substitute(
        cs: &[(Term, Term)],
        sub: &HashMap<Variable, Term>,
    ) -> Vec<(Term, Term)> {
        cs.iter()
            .map(|&(ref s, ref t)| (s.substitute(sub), t.substitute(sub)))
            .collect()
    }
    /// Compose two substitutions.
    fn compose(
        sub1: Option<HashMap<Variable, Term>>,
        sub2: Option<HashMap<Variable, Term>>,
    ) -> Option<HashMap<Variable, Term>> {
        match (sub1, sub2) {
            (Some(mut s1), Some(s2)) => {
                for (k, v) in s2 {
                    let v = v.substitute(&s1);
                    s1.insert(k, v);
                }
                Some(s1)
            }
            _ => None,
        }
    }
    /// Compute the [alpha equivalence] for two `Term`s.
    ///
    /// [alpha equivalence]: https://en.wikipedia.org/wiki/Lambda_calculus#Alpha_equivalence
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term, Term, Variable};
    /// # use std::collections::{HashMap, HashSet};
    /// let mut sig = Signature::default();
    /// let s = sig.new_op(0, Some("S".to_string()));
    ///
    /// let t = parse_term(&mut sig, "S K y_ z_").expect("parse of S K y_ z_");
    /// let t2 = parse_term(&mut sig, "S K a_ b_").expect("parse of S K a_ b_");
    /// let t3 = parse_term(&mut sig, "S K y_").expect("parse of S K y_");
    ///
    /// let vars = sig.variables();
    /// let (y, z, a, b) = (vars[0].clone(), vars[1].clone(), vars[2].clone(), vars[3].clone());
    ///
    /// assert_eq!(y.display(), "y_".to_string());
    /// assert_eq!(z.display(), "z_".to_string());
    /// assert_eq!(a.display(), "a_".to_string());
    /// assert_eq!(b.display(), "b_".to_string());
    ///
    /// let mut expected_alpha: HashMap<Variable, Term> = HashMap::new();
    /// expected_alpha.insert(y, Term::Variable(a));
    /// expected_alpha.insert(z, Term::Variable(b));
    ///
    /// assert_eq!(Term::alpha(&t, &t2), Some(expected_alpha));
    ///
    /// assert_eq!(Term::alpha(&t, &t3), None);
    /// ```
    pub fn alpha(t1: &Term, t2: &Term) -> Option<HashMap<Variable, Term>> {
        if Term::pmatch(vec![(t2.clone(), t1.clone())]).is_some() {
            Term::pmatch(vec![(t1.clone(), t2.clone())])
        } else {
            None
        }
    }
    /// Returns whether two `Term`s are shape equivalent.
    ///
    /// Shape equivalence is where two `Term`s may not contain the same subterms, but they share the same structure(a.k.a. shape).
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_term, Term};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "S K y_ z_").expect("parse of S K y_ z_");
    /// let t2 = parse_term(&mut sig, "A B x_ w_").expect("parse of A B x_ w_");
    /// let t3 = parse_term(&mut sig, "S K y_").expect("parse of S K y_");
    ///
    /// assert!(Term::shape_equivalent(&t, &t2));
    ///
    /// assert!(!Term::shape_equivalent(&t, &t3));
    /// ```
    pub fn shape_equivalent(t1: &Term, t2: &Term) -> bool {
        let mut vmap = HashMap::new();
        let mut omap = HashMap::new();
        Term::se_helper(t1, t2, &mut vmap, &mut omap)
    }
    fn se_helper(
        t1: &Term,
        t2: &Term,
        vmap: &mut HashMap<Variable, Variable>,
        omap: &mut HashMap<Operator, Operator>,
    ) -> bool {
        match (t1, t2) {
            (&Term::Variable(ref v1), &Term::Variable(ref v2)) => {
                v2 == vmap.entry(v1.clone()).or_insert_with(|| v2.clone())
            }
            (
                &Term::Application {
                    op: ref op1,
                    args: ref args1,
                },
                &Term::Application {
                    op: ref op2,
                    args: ref args2,
                },
            ) => {
                op2 == omap.entry(op1.clone()).or_insert_with(|| op2.clone())
                    && args1
                        .iter()
                        .zip(args2)
                        .all(|(a1, a2)| Term::se_helper(a1, a2, vmap, omap))
            }
            _ => false,
        }
    }
    /// Given a vector of contraints, return a substitution which satisfies the constrants.
    /// If the constraints are not satisfiable, return `None`. Constraints are in the form of
    /// patterns, where substitutions are only considered for variables in the first term of each
    /// pair.
    ///
    /// For more information see [`Pattern Matching`].
    ///
    /// [`Pattern Matching`]: https://en.wikipedia.org/wiki/Pattern_matching
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// # use std::collections::{HashMap, HashSet};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "C(A)").expect("parse of C(A)");
    ///
    /// let t2 = parse_term(&mut sig, "C(x_)").expect("parse of C(x_)");
    ///
    /// let t3 = parse_term(&mut sig, "C(y_)").expect("parse of C(y_)");
    ///
    /// let t4 = parse_term(&mut sig, "A(x_)").expect("parse of A(x_)");
    ///
    /// assert_eq!(Term::pmatch(vec![(t, t2.clone())]), None);
    ///
    /// let mut expected_sub = HashMap::new();
    ///
    /// // maps variable x in term t2 to variable y in term t3
    /// expected_sub.insert(t2.variables()[0].clone(), Term::Variable(t3.variables()[0].clone()));
    ///
    /// assert_eq!(Term::pmatch(vec![(t2, t3.clone())]), Some(expected_sub));
    ///
    /// assert_eq!(Term::pmatch(vec![(t3, t4)]), None);
    /// ```
    pub fn pmatch(cs: Vec<(Term, Term)>) -> Option<HashMap<Variable, Term>> {
        Term::unify_internal(cs, Unification::Match)
    }
    /// Given a vector of contraints, return a substitution which satisfies the constrants.
    /// If the constraints are not satisfiable, return `None`.
    ///
    /// For more information see [`Unification`].
    ///
    /// [`Unification`]: https://en.wikipedia.org/wiki/Unification_(computer_science)
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Term, parse_term};
    /// # use std::collections::{HashMap, HashSet};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_term(&mut sig, "C(A)").expect("parse of C(A)");
    ///
    /// let t2 = parse_term(&mut sig, "C(x_)").expect("parse of C(x_)");
    ///
    /// let t3 = parse_term(&mut sig, "C(y_)").expect("parse of C(y_)");
    ///
    /// let t4 = parse_term(&mut sig, "B(x_)").expect("parse of B(x_)");
    ///
    /// let mut expected_sub = HashMap::new();
    ///
    /// // maps variable x in term t2 to constant A in term t
    /// expected_sub.insert(
    ///     t2.variables()[0].clone(),
    ///     Term::Application {
    ///         op: t.operators()[0].clone(),
    ///         args:vec![],
    ///     },
    /// );
    ///
    /// assert_eq!(Term::unify(vec![(t, t2.clone())]), Some(expected_sub));
    ///
    /// let mut expected_sub = HashMap::new();
    ///
    ///  // maps variable x in term t2 to variable y in term t3
    /// expected_sub.insert(t2.variables()[0].clone(), Term::Variable(t3.variables()[0].clone()));
    ///
    /// assert_eq!(Term::unify(vec![(t2, t3.clone())]), Some(expected_sub));
    ///
    /// assert_eq!(Term::unify(vec![(t3, t4)]), None);
    /// ```
    pub fn unify(cs: Vec<(Term, Term)>) -> Option<HashMap<Variable, Term>> {
        Term::unify_internal(cs, Unification::Unify)
    }
    /// the internal implementation of unify and match.
    fn unify_internal(
        mut cs: Vec<(Term, Term)>,
        utype: Unification,
    ) -> Option<HashMap<Variable, Term>> {
        let c = cs.pop();
        match c {
            None => Some(HashMap::new()),
            Some((ref s, ref t)) if s == t => Term::unify_internal(cs, utype),
            Some((
                Term::Application {
                    op: ref h1,
                    args: ref a1,
                },
                Term::Application {
                    op: ref h2,
                    args: ref a2,
                },
            )) if h1 == h2 => {
                cs.append(&mut a1.clone().into_iter().zip(a2.clone().into_iter()).collect());
                Term::unify_internal(cs, utype)
            }
            Some((Term::Variable(ref var), ref t)) if !t.variables().contains(&&var) => {
                let mut st = HashMap::new();
                st.insert(var.clone(), t.clone());
                let mut cs = Term::constraint_substitute(&cs, &st);
                Term::compose(Term::unify_internal(cs, utype), Some(st))
            }
            Some((ref s, Term::Variable(ref var)))
                if !s.variables().contains(&&var) && utype != Unification::Match =>
            {
                let mut ts = HashMap::new();
                ts.insert(var.clone(), s.clone());
                let mut cs = Term::constraint_substitute(&cs, &ts);
                Term::compose(Term::unify_internal(cs, utype), Some(ts))
            }
            _ => None,
        }
    }
}
