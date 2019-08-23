%% -*- coding: utf-8 -*-
%% Automatically generated, do not edit
%% Generated by gpb_compile version 4.10.1

-ifndef(benchmarks_pb).
-define(benchmarks_pb, true).

-define(benchmarks_pb_gpb_version, "4.10.1").

-ifndef('PINGPONGREQUEST_PB_H').
-define('PINGPONGREQUEST_PB_H', true).
-record('PingPongRequest',
        {number_of_messages = 0 :: non_neg_integer() | undefined % = 1, 32 bits
        }).
-endif.

-ifndef('THROUGHPUTPINGPONGREQUEST_PB_H').
-define('THROUGHPUTPINGPONGREQUEST_PB_H', true).
-record('ThroughputPingPongRequest',
        {messages_per_pair = 0  :: non_neg_integer() | undefined, % = 1, 32 bits
         pipeline_size = 0      :: non_neg_integer() | undefined, % = 2, 32 bits
         parallelism = 0        :: non_neg_integer() | undefined, % = 3, 32 bits
         static_only = false    :: boolean() | 0 | 1 | undefined % = 4
        }).
-endif.

-ifndef('TESTRESULT_PB_H').
-define('TESTRESULT_PB_H', true).
-record('TestResult',
        {sealed_value           :: {success, benchmarks_pb:'TestSuccess'()} | {failure, benchmarks_pb:'TestFailure'()} | {not_implemented, benchmarks_pb:'NotImplemented'()} | undefined % oneof
        }).
-endif.

-ifndef('TESTSUCCESS_PB_H').
-define('TESTSUCCESS_PB_H', true).
-record('TestSuccess',
        {number_of_runs = 0     :: non_neg_integer() | undefined, % = 1, 32 bits
         run_results = []       :: [float() | integer() | infinity | '-infinity' | nan] | undefined % = 2
        }).
-endif.

-ifndef('TESTFAILURE_PB_H').
-define('TESTFAILURE_PB_H', true).
-record('TestFailure',
        {reason = []            :: iodata() | undefined % = 1
        }).
-endif.

-ifndef('NOTIMPLEMENTED_PB_H').
-define('NOTIMPLEMENTED_PB_H', true).
-record('NotImplemented',
        {
        }).
-endif.

-ifndef('READYREQUEST_PB_H').
-define('READYREQUEST_PB_H', true).
-record('ReadyRequest',
        {
        }).
-endif.

-ifndef('READYRESPONSE_PB_H').
-define('READYRESPONSE_PB_H', true).
-record('ReadyResponse',
        {status = false         :: boolean() | 0 | 1 | undefined % = 1
        }).
-endif.

-ifndef('SHUTDOWNREQUEST_PB_H').
-define('SHUTDOWNREQUEST_PB_H', true).
-record('ShutdownRequest',
        {force = false          :: boolean() | 0 | 1 | undefined % = 1
        }).
-endif.

-ifndef('SHUTDOWNACK_PB_H').
-define('SHUTDOWNACK_PB_H', true).
-record('ShutdownAck',
        {
        }).
-endif.

-endif.