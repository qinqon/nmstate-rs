# Copyright 2021 Red Hat
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import ctypes
from ctypes import (
    c_int,
    c_char_p,
    c_uint32,
    Structure,
    POINTER,
    byref,
)
from ctypes.util import find_library
import json

lib = ctypes.cdll.LoadLibrary("libnmstate.so.0")

lib.nmstate_net_state_retrieve.restype = c_int
lib.nmstate_net_state_retrieve.argtypes = (
    c_uint32,
    POINTER(c_char_p),
    POINTER(c_char_p),
    POINTER(c_char_p),
    POINTER(c_char_p),
)

lib.nmstate_err_kind_free.restype = None
lib.nmstate_err_kind_free.argtypes = (c_char_p,)
lib.nmstate_err_msg_free.restype = None
lib.nmstate_err_msg_free.argtypes = (c_char_p,)
lib.nmstate_log_free.restype = None
lib.nmstate_log_free.argtypes = (c_char_p,)
lib.nmstate_net_state_free.restype = None
lib.nmstate_net_state_free.argtypes = (c_char_p,)

NMSTATE_FLAG_NONE = 0
NMSTATE_FLAG_KERNEL_ONLY = 1 << 1
NMSTATE_PASS = 0


class NmstateError(Exception):
    def __init__(self, kind, msg):
        self.kind = kind
        self.msg = msg
        super().__init__(f"{kind}: {msg}")


def retrieve_net_state_json(kernel_only=False):
    c_err_msg = c_char_p()
    c_err_kind = c_char_p()
    c_state = c_char_p()
    c_log = c_char_p()
    flags = NMSTATE_FLAG_NONE
    if kernel_only:
        flags |= NMSTATE_FLAG_KERNEL_ONLY

    rc = lib.nmstate_net_state_retrieve(
        flags,
        byref(c_state),
        byref(c_log),
        byref(c_err_kind),
        byref(c_err_msg),
    )
    state = c_state.value
    err_msg = c_err_msg.value
    err_kind = c_err_kind.value
    lib.nmstate_log_free(c_log)
    lib.nmstate_net_state_free(c_state)
    lib.nmstate_err_kind_free(c_err_kind)
    lib.nmstate_err_msg_free(c_err_msg)
    if rc != NMSTATE_PASS:
        raise NmstateError(err_kind, err_msg)
    return state.decode("utf-8")


def apply_net_state(state, kernel_only=False):
    c_err_msg = c_char_p()
    c_err_kind = c_char_p()
    c_state = c_char_p(json.dumps(state).encode("utf-8"))
    c_log = c_char_p()
    flags = NMSTATE_FLAG_NONE
    if kernel_only:
        flags |= NMSTATE_FLAG_KERNEL_ONLY

    rc = lib.nmstate_net_state_apply(
        flags,
        c_state,
        byref(c_log),
        byref(c_err_kind),
        byref(c_err_msg),
    )
    err_msg = c_err_msg.value
    err_kind = c_err_kind.value
    lib.nmstate_log_free(c_log)
    lib.nmstate_err_kind_free(c_err_kind)
    lib.nmstate_err_msg_free(c_err_msg)
    if rc != NMSTATE_PASS:
        raise NmstateError(err_kind, err_msg)
