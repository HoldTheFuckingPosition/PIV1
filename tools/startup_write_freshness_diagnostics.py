"""Fixed, nonsecret refusal predicates for future dedicated Write scopes.

No clock acquisition, threshold selection, refresh, retry or success-path I/O.
The caller retains the authenticated sample, exact policy and durable Store.
"""


def require_freshness(request_age, receive_age, request_limit, *, record, stop):
    request_ok = request_age <= request_limit
    receive_ok = receive_age <= 1.0
    if request_ok and receive_ok:
        return
    # Fixed booleans only; no exception text, sample data or signer output.
    # If durability fails, propagate it without proceeding to the effect.
    record({'schema': 'piv1-write-freshness-refusal-v1',
            'requestStartWithinLimit': request_ok,
            'receiveAgeWithinOneSecond': receive_ok})
    raise stop('fresh_sample_aged_HOLD')
