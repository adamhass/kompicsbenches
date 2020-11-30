-module(sized_throughput).

-behaviour(distributed_benchmark).

-export([
	msg_to_master_conf/1,
	new_master/0,
	new_client/0,
	master_setup/3,
	master_prepare_iteration/2,
	master_run_iteration/1,
	master_cleanup_iteration/3,
	client_setup/2,
	client_prepare_iteration/1,
	client_cleanup_iteration/2,
	source/7,
	generate_message/1,
	sink/2]).

-record(master_conf, {
	num_pairs = 0 :: integer(),
	msg_size = 0 :: integer(),
	batch_size = 0 :: integer(),
	batch_count = 0 :: integer()}).
-type master_conf() :: #master_conf{}.
-record(client_conf, {
	num_pairs = 0 :: integer(),
	batch_size = 0 :: integer()}).
-type client_conf() :: #client_conf{}.
-type client_data() :: [pid()].

-record(master_state, {
	config = #master_conf{} :: master_conf(),
	sources :: [pid()] | 'undefined',
	sinks :: [pid()] | 'undefined'}).

-type master_instance() :: #master_state{}.

-record(client_state, {
	config = #client_conf{} :: client_conf(),
	sinks :: [pid()] | 'undefined'}).

-type client_instance() :: #client_state{}.

-spec get_batch_count(State :: master_instance()) -> integer().
get_batch_count(State) ->
	State#master_state.config#master_conf.batch_count.

-spec get_num_pairs(State :: master_instance() | client_instance()) -> integer().
get_num_pairs(State) ->
	case State of
		#master_state{ config = Conf } ->
			Conf#master_conf.num_pairs;
		#client_state{ config = Conf } ->
			Conf#client_conf.num_pairs
	end.

-spec get_msg_size(State :: master_instance()) -> integer().
get_msg_size(State) ->
	State#master_state.config#master_conf.msg_size.

-spec get_batch_size(State :: master_instance() | client_instance()) -> integer().
get_batch_size(State) ->
	case State of
		#master_state{ config = Conf } ->
			Conf#master_conf.batch_size;
		#client_state{ config = Conf } ->
			Conf#client_conf.batch_size
	end.

-spec msg_to_master_conf(Msg :: term()) ->
	{ok, MasterConf :: master_conf()} |
	{error, Reason :: string()}.
msg_to_master_conf(Msg) ->
	case Msg of
		#{	message_size := MsgSize,
				batch_size := BatchSize,
				number_of_batches := BatchCount,
				number_of_pairs := NumPairs
        } when
        	is_integer(MsgSize) andalso (MsgSize > 0),
        	is_integer(BatchSize) andalso (BatchSize > 0),
        	is_integer(BatchCount) andalso (BatchCount > 0),
        	is_integer(NumPairs) andalso (NumPairs > 0) ->
			{ok, #master_conf{}};
		#{	message_size := _MsgSize,
				batch_size := _BatchSize,
				number_of_batches := _BatchCount,
				number_of_pairs := _NumPairs
        } ->
			{error, io_lib:fwrite("Invalid config parameters:~p.~n", [Msg])};
		_ ->
			{error, io_lib:fwrite("Invalid config message:~p.~n", [Msg])}
	end.

-spec new_master() -> MasterInstance :: master_instance().
new_master() ->
	#master_state{}.

-spec new_client() -> ClientInstance :: client_instance().
new_client() ->
	#client_state{}.

%%%% On Master Instance %%%%%

-spec master_setup(Instance :: master_instance(), Conf :: master_conf(), Meta :: distributed_benchmark:deployment_metadata()) ->
	{ok, Newnstance :: master_instance(), ClientConf :: client_conf()}.
master_setup(Instance, Conf, _Meta) ->
	NewInstance = Instance#master_state{config = Conf},
	process_flag(trap_exit, true),
	ClientConf = #client_conf{num_pairs = get_num_pairs(NewInstance), batch_size = get_batch_size(NewInstance) },
	{ok, NewInstance, ClientConf}.

-spec master_prepare_iteration(Instance :: master_instance(), ClientData :: [client_data()]) ->
	{ok, NewInstance :: master_instance()}.
master_prepare_iteration(Instance, ClientData) ->
	[Sinks| _Rest] = ClientData,
	Self = self(),
	if
		Instance#master_state.sources == undefined ->
			%% Preparing first iteration
			SourceFun = fun(Sink) ->
				source(Sink, generate_message(get_msg_size(Instance)), get_batch_size(Instance), get_batch_count(Instance), 0, 0, Self)
			end,
			Sources = lists:map(fun(Sink) -> spawn_link(SourceFun(Sink)) end, Sinks),
			NewInstance = Instance#master_state{sources = Sources, sinks = Sinks},
			{ok, NewInstance};
		true ->
			%% Already prepared, do nothing.
			{ok, Instance}
	end.

-spec master_run_iteration(Instance :: master_instance()) ->
	{ok, NewInstance :: master_instance()}.
master_run_iteration(Instance) ->
	lists:foreach(fun(Source) -> Source ! start end, Instance#master_state.sources),
	ok = bench_helpers:await_all(Instance#master_state.sources, ok),
	{ok, Instance}.

-spec master_cleanup_iteration(Instance :: master_instance(), LastIteration :: boolean(), ExecTimeMillis :: float()) ->
	{ok, NewInstance :: master_instance()}.
master_cleanup_iteration(Instance, LastIteration, _ExecTimeMillis) ->
	case LastIteration of
		true ->
			lists:foreach(fun(Source) -> Source ! stop end, Instance#master_state.sources),
			ok = bench_helpers:await_exit_all(Instance#master_state.sources),
			NewInstance = Instance#master_state{sources = undefined},
			{ok, NewInstance};
		_ ->
			{ok, Instance}
	end.

%%%% On Client Instance %%%%%

-spec client_setup(Instance :: client_instance(), Conf :: client_conf()) ->
	{ok, NewInstance :: client_instance(), ClientData :: client_data()}.
client_setup(Instance, Conf) ->
	ConfInstance = Instance#client_state{config = Conf},
	process_flag(trap_exit, true),
	Range = lists:seq(1, get_num_pairs(ConfInstance)),
	Sinks = lists:map(fun(_Index) -> spawn_link(sink(ConfInstance#client_conf.batch_size, 0)) end, Range),
	NewInstance = ConfInstance#client_state{sinks = Sinks},
	{ok, NewInstance, Sinks}.

-spec client_prepare_iteration(Instance :: client_instance()) ->
	{ok, NewInstance :: client_instance()}.
client_prepare_iteration(Instance) ->
	io:fwrite("Preparing ponger iteration.~n"),
	{ok, Instance}.

-spec client_cleanup_iteration(Instance :: client_instance(), LastIteration :: boolean()) ->
	{ok, NewInstance :: client_instance()}.
client_cleanup_iteration(Instance, LastIteration) ->
	io:fwrite("Cleaning up ponger side.~n"),
	case LastIteration of
		true ->
			lists:foreach(fun(Sink) -> Sink ! stop end, Instance#client_state.sinks),
			ok = bench_helpers:await_exit_all(Instance#client_state.sinks),
			NewInstance = Instance#client_state{sinks = undefined},
			{ok, NewInstance};
		_ ->
			{ok, Instance}
	end.

%%%%%% Source %%%%%%
-spec source(Sink :: pid(), Msg :: binary(), BatchSize :: integer(), BatchCount :: integer(), AckCount :: integer(), SentBatches :: integer(), Return :: pid()) -> ok.
source(Sink, Msg, BatchSize, BatchCount, AckCount, SentBatches, Return) ->
	if
		AckCount < BatchCount ->
			%% Waiting for start or ack
			receive
				start ->
					%io:fwrite("Starting source ~p.~n", [self()]),
					% Send two batches and then wait for acks
					send_msgs(Sink, Msg, BatchSize, 0),
					send_msgs(Sink, Msg, BatchSize, 0),
					source(Sink, Msg, BatchSize, BatchCount, AckCount, SentBatches+2, Return);
				ack ->
					if
						SentBatches < BatchCount ->
							% Send another batch and continue
							send_msgs(Sink, Msg, BatchSize, 0),
							source(Sink, Msg, BatchSize, BatchCount, AckCount+1, SentBatches+1, Return);
						true ->
							% Sent all batches, only waiting for the last acks.
							source(Sink, Msg, BatchSize, BatchCount, AckCount+1, SentBatches, Return)
					end;
				stop ->
					ok;
				X ->
					io:fwrite("Source got unexpected message: ~p!~n",[X]),
					throw(X) % don't accept weird stuff
			end;
		true ->
			%% all acks received send ok and await next start or stop
			Return ! {ok},
			source(Sink, Msg, BatchSize, BatchCount, 0, 0, Return)
		end.

-spec send_msgs(Sink :: pid(), Msg :: binary(), BatchSize :: integer(), SentMessages :: integer()) -> ok.
send_msgs(Sink, Msg, BatchSize, SentMessages) ->
	if
	SentMessages < BatchSize ->
		Sink ! {message, Msg, 1, self()},
		send_msgs(Sink, Msg, BatchSize, SentMessages + 1);
	true ->
		ok
	end.

generate_message(Size) when is_integer(Size) ->
	random_bytes = crypto:strong_rand_bytes(Size).

%%%%%% Sink %%%%%%
-spec sink(BatchSize :: integer(), Received :: integer()) -> ok.
sink(BatchSize, Received) ->
	receive
		stop ->
			%io:fwrite("Stopping ponger ~w.~n", [self()]),
			ok;
		{message, _, Aux, Source} ->
			if
				Received + 1 == BatchSize ->
					Source ! ack,
					sink(BatchSize, 0);
				true ->
					sink(BatchSize, Received+Aux)
			end;
		X ->
			io:fwrite("Ponger got unexpected message: ~p!~n",[X]),
			throw(X) % don't accept weird stuff
	end.
