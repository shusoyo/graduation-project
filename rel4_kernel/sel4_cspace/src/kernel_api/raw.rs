use sel4_common::structures::exception_t;
use vstd::prelude::*;

verus! {

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExException(exception_t);

pub uninterp spec fn is_exception_none(status: exception_t) -> bool;

pub uninterp spec fn is_exception_lookup_fault(status: exception_t) -> bool;

pub uninterp spec fn is_exception_syscall_error(status: exception_t) -> bool;

pub uninterp spec fn spec_runtime_exception_none() -> exception_t;

#[verifier::external_body]
pub fn runtime_exception_none() -> (ret: exception_t)
    ensures
        is_exception_none(ret),
        ret == spec_runtime_exception_none(),
{
    exception_t::EXCEPTION_NONE
}

#[verifier::external_body]
pub fn runtime_exception_lookup_fault() -> (ret: exception_t)
    ensures
        is_exception_lookup_fault(ret),
{
    exception_t::EXCEPTION_LOOKUP_FAULT
}

#[verifier::external_body]
pub fn runtime_status_is_lookup_fault(status: exception_t) -> (ret: bool)
    ensures
        ret == is_exception_lookup_fault(status),
{
    status == exception_t::EXCEPTION_LOOKUP_FAULT
}

#[verifier::external_body]
pub fn runtime_status_is_none(status: exception_t) -> (ret: bool)
    ensures
        ret == (status == spec_runtime_exception_none()),
{
    status == exception_t::EXCEPTION_NONE
}

#[verifier::external_body]
pub proof fn lemma_exception_none_not_lookup_fault(status: exception_t)
    requires
        is_exception_none(status),
    ensures
        !is_exception_lookup_fault(status),
{
}

#[verifier::external_body]
pub proof fn lemma_exception_lookup_fault_not_none(status: exception_t)
    requires
        is_exception_lookup_fault(status),
    ensures
        !is_exception_none(status),
{
}

#[verifier::external_body]
pub proof fn lemma_exception_syscall_error_not_none(status: exception_t)
    requires
        is_exception_syscall_error(status),
    ensures
        !is_exception_none(status),
{
}

#[verifier::external_body]
pub proof fn lemma_exception_none_not_syscall_error(status: exception_t)
    requires
        is_exception_none(status),
    ensures
        !is_exception_syscall_error(status),
{
}

#[verifier::external_body]
pub proof fn lemma_spec_runtime_exception_none_is_none()
    ensures
        is_exception_none(spec_runtime_exception_none()),
{
}

#[verifier::external_body]
pub proof fn lemma_exception_none_iff_spec_runtime_exception_none(status: exception_t)
    ensures
        is_exception_none(status) <==> status == spec_runtime_exception_none(),
{
}

#[verifier::external_body]
pub fn runtime_exception_syscall_error() -> (ret: exception_t)
    ensures
        is_exception_syscall_error(ret),
{
    exception_t::EXCEPTION_SYSCALL_ERROR
}

}
