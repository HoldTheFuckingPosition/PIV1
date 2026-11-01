"""Nonsecret failure metadata for the dedicated Write caller, before its HOLD.

The shared signer suppresses display of the original exception but retains its
in-memory __context__. Never use this to infer a historical, unavailable cause.
Location means the public wrapper propagation site, not a backend traceback.
"""

_CODES = frozenset((
    'fresh_sample_aged_HOLD', 'fresh_original_40heights',
    'signer_timeout', 'signer_output_bound', 'signer_backend_refused',
    'one_native_Popen_admission',
))
_TYPES = ((ValueError, 'ValueError'), (TypeError, 'TypeError'),
          (TimeoutError, 'TimeoutError'), (OSError, 'OSError'),
          (PermissionError, 'PermissionError'),
          (FileNotFoundError, 'FileNotFoundError'),
          (BrokenPipeError, 'BrokenPipeError'))


def describe(failure, boundary_code, operator_stop, admission_stop):
    """Use exact trusted types/frame identity; never format exception values."""
    result = {'schema': 'piv1-signer-error-diagnostic-v1',
              'exceptionType': 'REDACTED', 'errorCode': 'CAUSE_UNAVAILABLE',
              'wrapperCallsite': {'file': 'tools/startup_release_operator.py',
                                  'function': 'invoke_signer', 'line': None}}
    if type(failure) is not operator_stop:
        return result
    args = BaseException.args.__get__(failure)
    if len(args) != 1 or type(args[0]) is not str or args[0] != 'signer_attempt_unresolved':
        return result
    tb = BaseException.__traceback__.__get__(failure)
    frame = None
    while tb is not None:
        if tb.tb_frame.f_code is boundary_code:
            frame = tb.tb_frame
            break
        tb = tb.tb_next
    cause = BaseException.__context__.__get__(failure)
    if frame is None or cause is None:
        return result
    tb = BaseException.__traceback__.__get__(cause)
    while tb is not None and tb.tb_frame is not frame:
        tb = tb.tb_next
    if tb is None:
        return result  # An outer/stale exception is not this invocation's cause.
    result['wrapperCallsite']['line'] = tb.tb_lineno
    result['errorCode'] = 'UNCLASSIFIED_SIGNER_FAILURE'
    allowed = ((operator_stop, 'Stop'), (admission_stop, 'Stop')) + _TYPES
    name = next((name for cls, name in allowed if type(cause) is cls), None)
    if name is None:
        return result
    result['exceptionType'] = name
    args = BaseException.args.__get__(cause)
    if len(args) == 1 and type(args[0]) is str and args[0] in _CODES:
        result['errorCode'] = args[0]
    return result


def invoke(operator, adapter, payload, *, raw_capture, admission_stop,
           store, phase, encode):
    """Capture through the existing durable Store before the caller's HOLD."""
    try:
        return operator.invoke_signer(adapter, payload, _raw_capture=raw_capture)
    except operator.Stop as failure:
        row = describe(failure, operator.invoke_signer.__code__,
                       operator.Stop, admission_stop)
        store.capture(lambda: store.write(f'phase-{phase:04}-native-error.json', encode(row)))
        raise
