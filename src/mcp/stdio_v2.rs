//! Explicit /2 dispatch retains the same accepted controller and genuine owner through stdout.
//! The original /1 worker/driver bodies remain unchanged.

use super::super::native_sources_v2::{self, ServerReadGateV2};
use super::{
    Accepted, Catalog, Clock, Control, Framer, IDLE_WAIT, InputPort, Machine, Operation, Pump,
    Query, QueryError, READ_BURST, Reason, RequestId, RunError, Startup, SystemClock, WorkError,
    Write, execute, queries, stopped, unavailable,
};

/// Run the selected /2 family without calling any /1 disclosure constructor.
pub(super) fn run<P: InputPort, W: Write>(
    startup: &Startup,
    input: P,
    output: W,
) -> Result<(), RunError> {
    run_with_clock(startup, input, output, &SystemClock)
}
/// Same actual accepted deadlines, pump and bounded static catalog as the maintained worker.
fn run_with_clock<P: InputPort, W: Write>(
    startup: &Startup,
    input: P,
    output: W,
    clock: &dyn Clock,
) -> Result<(), RunError> {
    let catalog = Catalog::new_v2().map_err(|_| RunError::Catalog)?;
    let mut pump = Pump {
        input,
        output,
        catalog: &catalog,
        clock,
        machine: Machine::default(),
        framer: Framer::default(),
        intake: [0; READ_BURST],
        start: 0,
        end: 0,
    };
    loop {
        if pump.machine.shutdown {
            return Ok(());
        }
        if let Some(accepted) = pump.machine.next() {
            let id = accepted.id.clone();
            let mut control = Control { pump: &mut pump, io_failure: None, quiescent: false };
            let result = execute_v2(startup, &catalog, accepted, &mut control);
            let failure = control.io_failure;
            // execute has returned: every PreparedQuery and scope is now actually dropped.
            control.pump.machine.finish(&id);
            if let Some(error) = failure {
                return Err(error);
            }
            result?;
        } else if let Err(error) = pump.service(IDLE_WAIT) {
            pump.machine.shutdown();
            return Err(error);
        }
    }
}

/// Static operations reuse the unchanged path; only tools/call selects the /2 native owner.
fn execute_v2<P: InputPort, W: Write>(
    startup: &Startup,
    catalog: &Catalog,
    accepted: Accepted,
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    match accepted.operation {
        Operation::Call(query) => call_v2(startup, catalog, &accepted.id, &query, control),
        operation => execute(
            startup,
            catalog,
            Accepted { id: accepted.id, operation, deadline: accepted.deadline },
            control,
        ),
    }
}
/// Plain outcome is handled only after the actual generic owner and all output tickets unwind.
enum Outcome {
    /// No factory success existed; use the fixed null response on the original worker.
    Unavailable,
    /// Original typed sticky stop, never downgraded by a later domain error.
    Work(WorkError),
    /// Actual publication result after complete owner/output verification.
    Published(Result<(), RunError>),
}
/// Keep external control borrowing outside the retained owner's entire inner lifetime.
fn call_v2<P: InputPort, W: Write>(
    startup: &Startup,
    catalog: &Catalog,
    id: &RequestId,
    query: &Query,
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    let outcome = owned_call(startup, catalog, id, query, control);
    match outcome {
        Outcome::Unavailable => {
            unavailable(catalog, id, query, Reason::ApprovalUnavailable, control, false)
        }
        Outcome::Work(error) => stopped(catalog, id, query, error, control),
        Outcome::Published(result) => result,
    }
}
/// Genuine capture/query/encoding/validation/final fences all borrow the same original controller.
fn owned_call<P: InputPort, W: Write>(
    startup: &Startup,
    catalog: &Catalog,
    id: &RequestId,
    query: &Query,
    control: &mut Control<'_, '_, P, W>,
) -> Outcome {
    let (Some(root), Some(pin)) =
        (startup.decision_root.as_deref(), startup.decision_sha256.as_deref())
    else {
        return Outcome::Unavailable;
    };
    let gate = native_sources_v2::capture_server_decision(
        &startup.project_root,
        root,
        pin,
        &startup.profile_sha256,
        control,
    );
    let scope = match gate {
        Ok(ServerReadGateV2::Validated(scope)) => scope,
        Ok(ServerReadGateV2::Unavailable) => return Outcome::Unavailable,
        Err(error) => return Outcome::Work(error),
    };
    let prepared = queries::prepare_v2(&scope, query);
    let unavailable_response;
    let (response, is_error) = match prepared.as_ref() {
        Ok(prepared) => (prepared.response(), false),
        Err(QueryError::Work(_)) => {
            return match prepared {
                Err(QueryError::Work(error)) => Outcome::Work(error),
                _ => unreachable!(),
            };
        }
        Err(QueryError::Unavailable(reason)) => {
            // Query refusal with a genuine owner cannot use the no-owner fallback.
            if let Err(error) = scope.verify_inputs() {
                return Outcome::Work(error);
            }
            unavailable_response = queries::unavailable(query, *reason);
            (&unavailable_response, true)
        }
    };
    let encoded = match queries::encode_v2(&scope, response, id, catalog, is_error) {
        Ok(encoded) => encoded,
        Err(QueryError::Work(error)) => return Outcome::Work(error),
        Err(QueryError::Unavailable(reason)) => {
            if let Err(error) = scope.verify_inputs() {
                return Outcome::Work(error);
            }
            let null = queries::unavailable(query, reason);
            match queries::encode_v2(&scope, &null, id, catalog, true) {
                Ok(encoded) => encoded,
                Err(QueryError::Work(error)) => return Outcome::Work(error),
                Err(QueryError::Unavailable(_)) => {
                    return Outcome::Published(Err(RunError::Encoding));
                }
            }
        }
    };
    // Never hold the controller RefMut while calling owner/admission/freshness methods.
    if let Err(error) = scope.with_control(Control::enter_publication) {
        scope.with_control(|control| control.quiescent = false);
        return Outcome::Work(error);
    }
    let final_check = match prepared.as_ref() {
        Ok(prepared) => prepared.verify_inputs(),
        Err(_) => scope.verify_inputs(),
    };
    if let Err(error) = final_check {
        scope.with_control(|control| control.quiescent = false);
        return Outcome::Work(error);
    }
    match scope.with_control(|control| control.publish(encoded.value())) {
        Ok(None) => Outcome::Published(Ok(())),
        Ok(Some(reason)) => Outcome::Work(WorkError::Interrupted(reason)),
        Err(error) => Outcome::Published(Err(error)),
    }
}
