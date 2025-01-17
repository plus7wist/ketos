//! Provides facilities for expressing and storing tracebacks.
//!
//! When a compilation or execution operation returns an error, it will store
//! a traceback within thread-local storage. To access this value, the
//! functions `get_traceback` (which clones the value) and `take_traceback`
//! (which removes the value) can be used.

use std::cell::RefCell;
use std::fmt::{self, Write};
use std::mem::replace;

use crate::name::{Name, NameDisplay, NameStore};
use crate::pretty::pretty_print;
use crate::value::Value;

/// Represents a series of items, beginning with the outermost context
/// and culminating with the context in which an error was generated.
#[derive(Clone)]
pub struct Trace {
	items: Vec<TraceItem>,
	expr: Option<Value>,
}

impl Trace {
	/// Creates a new `Trace` from a series of items.
	pub fn new(items: Vec<TraceItem>, expr: Option<Value>) -> Trace {
		Trace { items, expr }
	}

	/// Creates a new `Trace` from a single item.
	pub fn single(item: TraceItem, expr: Option<Value>) -> Trace {
		Trace::new(vec![item], expr)
	}

	/// Returns the series of traced items.
	pub fn items(&self) -> &[TraceItem] {
		&self.items
	}

	/// Returns a borrowed reference to the optional contained expression.
	pub fn expr(&self) -> Option<&Value> {
		self.expr.as_ref()
	}

	/// Takes the optional contained expression and returns it.
	pub fn take_expr(&mut self) -> Option<Value> {
		self.expr.take()
	}
}

thread_local!(static TRACEBACK: RefCell<Option<Trace>> = RefCell::new(None));

/// Removes the traceback value for the current thread.
pub fn clear_traceback() {
	TRACEBACK.with(|tb| *tb.borrow_mut() = None);
}

/// Clones and returns the traceback value for the current thread.
///
/// The value remains stored for future calls to `get_traceback`.
pub fn get_traceback() -> Option<Trace> {
	TRACEBACK.with(|tb| tb.borrow().clone())
}

/// Assigns a traceback value for the current thread.
pub fn set_traceback(trace: Trace) {
	TRACEBACK.with(|tb| *tb.borrow_mut() = Some(trace));
}

/// Removes and returns the traceback value for the current thread.
pub fn take_traceback() -> Option<Trace> {
	TRACEBACK.with(|tb| replace(&mut *tb.borrow_mut(), None))
}

/// Represents a single traceable event in either compilation or
/// execution of code.
#[derive(Copy, Clone, Debug)]
pub enum TraceItem {
	/// Call to a code object; `(scope name, code name)`
	CallCode(Name, Name),
	/// Call to a code object generated by an expression
	CallExpr(Name),
	/// Call to an anonymous function
	CallLambda(Name),
	/// Call to a macro; `(scope name, macro name)`
	CallMacro(Name, Name),
	/// Expansion of an operator; `(scope name, operator name)`
	CallOperator(Name, Name),
	/// Call to a system function
	CallSys(Name),
	/// Definition of a named value; `(scope name, definition name)`
	Define(Name, Name),
	/// Definition of a constant value; `(scope name, const name)`
	DefineConst(Name, Name),
	/// Definition of an anonymous lambda
	DefineLambda(Name),
	/// Definition of a macro; `(scope name, macro name)`
	DefineMacro(Name, Name),
	/// Definition of a structure; `(scope name, struct name)`
	DefineStruct(Name, Name),
	/// Module import declaration; `(scope name, module name)`
	UseModule(Name, Name),
}

impl NameDisplay for Trace {
	fn fmt(&self, names: &NameStore, f: &mut fmt::Formatter) -> fmt::Result {
		use self::TraceItem::*;

		for item in &self.items {
			match *item {
				CallCode(m, n) => writeln!(f, "  In {}, function {}", names.get(m), names.get(n))?,
				CallExpr(m) => writeln!(f, "  In {}, call expression", names.get(m))?,
				CallLambda(m) => writeln!(f, "  In {}, lambda", names.get(m))?,
				CallMacro(m, n) => {
					writeln!(f, "  In {}, macro expansion {}", names.get(m), names.get(n))?
				}
				CallOperator(m, n) => {
					writeln!(f, "  In {}, operator {}", names.get(m), names.get(n))?
				}
				CallSys(n) => writeln!(f, "  In system function {}", names.get(n))?,
				Define(m, n) => writeln!(f, "  In {}, define {}", names.get(m), names.get(n))?,
				DefineConst(m, n) => writeln!(f, "  In {}, const {}", names.get(m), names.get(n))?,
				DefineLambda(m) => writeln!(f, "  In {}, lambda", names.get(m))?,
				DefineMacro(m, n) => writeln!(f, "  In {}, macro {}", names.get(m), names.get(n))?,
				DefineStruct(m, n) => {
					writeln!(f, "  In {}, struct {}", names.get(m), names.get(n))?
				}
				UseModule(m, n) => writeln!(f, "  In {}, use {}", names.get(m), names.get(n))?,
			}
		}

		if let Some(ref expr) = self.expr {
			f.write_str("    ")?;
			pretty_print(f, names, expr, 4)?;
			f.write_char('\n')?;
		}

		Ok(())
	}
}
