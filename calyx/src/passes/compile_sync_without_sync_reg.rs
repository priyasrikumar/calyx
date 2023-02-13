use crate::errors::{CalyxResult, Error};
use crate::ir::traversal::{Action, Named, VisResult, Visitor};
use crate::ir::RRC;
use crate::ir::{self, GetAttributes};
use crate::{build_assignments, guard, structure};
use std::collections::HashMap;

#[derive(Default)]
pub struct CompileSyncWithoutSyncReg;

impl Named for CompileSyncWithoutSyncReg {
    fn name() -> &'static str {
        "compile-sync-without-sync-reg"
    }

    fn description() -> &'static str {
        "Implement barriers for statements marked with @sync attribute without
      std_sync_ref"
    }
}

type BarrierMap = HashMap<u64, (RRC<ir::Cell>, Box<ir::Guard>)>;

trait Gettable {
    fn get_reg(&mut self, idx: &u64) -> &mut RRC<ir::Cell>;

    fn get_guard(&mut self, idx: &u64) -> &mut Box<ir::Guard>;
}

impl Gettable for BarrierMap {
    fn get_reg(&mut self, idx: &u64) -> &mut RRC<ir::Cell> {
        let (cell, _) = self.get_mut(idx).unwrap();
        cell
    }

    fn get_guard(&mut self, idx: &u64) -> &mut Box<ir::Guard> {
        let (_, gd) = self.get_mut(idx).unwrap();
        gd
    }
}

fn build_barrier_group(
    builder: &mut ir::Builder,
    barrier_idx: &u64,
    barrier_reg: &mut BarrierMap,
) -> ir::Control {
    let group = builder.add_group("barrier");
    structure!(
        builder;
        let bar = prim std_reg(1);
        let z = constant(0, 1);
        let constant = constant(1, 1);
    );

    let g = barrier_reg.get_guard(barrier_idx);
    g.update(|g| g.and(guard!(bar["out"])));
    drop(g);

    let s = barrier_reg.get_reg(barrier_idx);

    let assigns = build_assignments!(builder;
        bar["in"] = ? constant["out"];
        bar["write_en"] = ? constant["out"];
        group["done"] = ? s["out"];
    );
    group.borrow_mut().assignments.extend(assigns);

    let clear = builder.add_group("clear");
    let clear_assigns = build_assignments!(builder;
        bar["in"] = ? z["out"];
        bar["write_en"] = ? constant["out"];);
    clear.borrow_mut().assignments.extend(clear_assigns);

    let stmts = vec![ir::Control::enable(group), ir::Control::enable(clear)];

    ir::Control::seq(stmts)
}

fn produce_err(con: &ir::Control) -> CalyxResult<()> {
    match con {
        ir::Control::Enable(e) => {
            if con.get_attributes().get("sync").is_some() {
                return Err(Error::malformed_control(
                    "Enable or Invoke controls cannot be marked with @sync"
                        .to_string(),
                )
                .with_pos(e.get_attributes()));
            }
            Ok(())
        }
        ir::Control::Invoke(i) => {
            if con.get_attributes().get("sync").is_some() {
                return Err(Error::malformed_control(
                    "Enable or Invoke controls cannot be marked with @sync"
                        .to_string(),
                )
                .with_pos(&i.attributes));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn insert_barrier(
    builder: &mut ir::Builder,
    con: &mut ir::Control,
    barrier_reg: &mut BarrierMap,
    barrier_con: &mut HashMap<u64, ir::Control>,
) -> CalyxResult<()> {
    match con {
        ir::Control::Empty(_) => {
            if let Some(&n) = con.get_attributes().get("sync") {
                if barrier_reg.get(&n).is_none() {
                    structure!(builder;
                        let s = prim std_reg(1);
                    );
                    let gd = ir::Guard::True;
                    barrier_reg.insert(n, (s, Box::new(gd)));
                }
                let con_ref = barrier_con.entry(n).or_insert_with(|| {
                    build_barrier_group(builder, &n, barrier_reg)
                });
                std::mem::swap(con, &mut ir::Cloner::control(con_ref));
            }
            Ok(())
        }
        ir::Control::Seq(seq) => {
            for s in seq.stmts.iter_mut() {
                insert_barrier(builder, s, barrier_reg, barrier_con)?;
            }
            Ok(())
        }
        ir::Control::If(i) => {
            insert_barrier(builder, &mut i.tbranch, barrier_reg, barrier_con)?;
            insert_barrier(builder, &mut i.fbranch, barrier_reg, barrier_con)?;
            Ok(())
        }
        ir::Control::While(w) => {
            insert_barrier(builder, &mut w.body, barrier_reg, barrier_con)?;
            Ok(())
        }
        ir::Control::Enable(_) | ir::Control::Invoke(_) => {
            produce_err(con)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
impl Visitor for CompileSyncWithoutSyncReg {
    fn finish_par(
        &mut self,
        s: &mut ir::Par,
        comp: &mut ir::Component,
        sigs: &ir::LibrarySignatures,
        _comps: &[ir::Component],
    ) -> VisResult {
        let mut builder = ir::Builder::new(comp, sigs);
        let mut barrier_reg: BarrierMap = HashMap::new();
        for stmt in s.stmts.iter_mut() {
            let mut barrier_con: HashMap<u64, ir::Control> = HashMap::new();
            insert_barrier(
                &mut builder,
                stmt,
                &mut barrier_reg,
                &mut barrier_con,
            )?;
        }

        for (_, (reg, g_box)) in barrier_reg {
            structure!( builder;
                let constant = constant(1,1);
            );
            let g = *g_box;
            let cont_assigns = build_assignments!(builder;
                reg["in"] = g ? constant["out"];
                reg["write_en"] = ? constant["out"];
            );
            builder
                .component
                .continuous_assignments
                .extend(cont_assigns);
        }
        Ok(Action::Continue)
    }
}
