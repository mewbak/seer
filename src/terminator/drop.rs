use rustc::mir;
use rustc::ty::{self, Ty};
use syntax::codemap::Span;

use error::EvalResult;
use eval_context::{EvalContext, StackPopCleanup, ValTy};
use place::{Place, PlaceExtra};
use value::PrimVal;
use value::Value;

impl<'a, 'tcx> EvalContext<'a, 'tcx> {
    pub(crate) fn drop_place(&mut self, lval: Place<'tcx>, instance: ty::Instance<'tcx>, ty: Ty<'tcx>, span: Span) -> EvalResult<'tcx> {
        trace!("drop_place: {:#?}", lval);

        // FIXME: Surely there is a more robust  way to check for this case?
        if format!("{:?}", ty) == "std::io::Stdin" {
            return Ok(())
        }

        let val = match self.force_allocation(lval)? {
            Place::Ptr { ptr, extra: PlaceExtra::Vtable(vtable) } => Value::ByValPair(ptr, PrimVal::Ptr(vtable)),
            Place::Ptr { ptr, extra: PlaceExtra::Length(len) } => Value::ByValPair(ptr, len),
            Place::Ptr { ptr, extra: PlaceExtra::None } => Value::ByVal(ptr),
            _ => bug!("force_allocation broken"),
        };
        self.drop(val, instance, ty, span)
    }
    pub(crate) fn drop(&mut self, arg: Value, mut instance: ty::Instance<'tcx>, ty: Ty<'tcx>, span: Span) -> EvalResult<'tcx> {
        trace!("drop: {:#?}, {:?}, {:?}", arg, ty.sty, instance.def);

        if let ty::InstanceDef::DropGlue(_, None) = instance.def {
            trace!("nothing to do, aborting");
            // we don't actually need to drop anything
            return Ok(());
        }
        let mir = match ty.sty {
            ty::TyDynamic(..) => {
                let vtable = match arg {
                    Value::ByValPair(_, PrimVal::Ptr(vtable)) => vtable,
                    _ => bug!("expected fat ptr, got {:?}", arg),
                };
                match self.read_drop_type_from_vtable(vtable)? {
                    Some(func) => {
                        instance = func;
                        self.load_mir(func.def)?
                    },
                    // no drop fn -> bail out
                    None => return Ok(()),
                }
            },
            _ => self.load_mir(instance.def)?,
        };

        self.push_stack_frame(
            instance,
            span,
            mir,
            Place::undef(),
            StackPopCleanup::None,
        )?;

        let mut arg_locals = self.frame().mir.args_iter();
        assert_eq!(self.frame().mir.arg_count, 1);
        let arg_local = arg_locals.next().unwrap();
        let dest = self.eval_place(&mir::Place::Local(arg_local))?;
        let arg_ty = self.tcx.mk_mut_ptr(ty);
        self.write_value(ValTy { value: arg, ty: arg_ty }, dest)
    }
}
