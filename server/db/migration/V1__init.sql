create domain reference_id as text
check (
  value ~ '^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$'
  and
  1 <= length(value)
  and
  length(value) <= 200
);

create table hub(
  id         uuid        not null default(gen_random_uuid()),
  created_at timestamptz not null default(now() at time zone 'utc'),
  updated_at timestamptz,
  name       text        not null,

  constraint pk_hub primary key(id),
  constraint ck_name check(
    1 <= length(name) 
    and 
    length(name) <= 200
  )
);

create unique index ix_hub__name on hub(lower(name));

create table thing(
  id             uuid         not null default(gen_random_uuid()),
  created_at     timestamptz  not null default(now() at time zone 'utc'),
  updated_at     timestamptz, 
  reference_id   reference_id not null,
  reported_by_id uuid         not null,

  constraint pk_thing                               primary key(id),
  constraint fk_reported_by_id                      foreign key(reported_by_id) references hub(id),
  constraint uq_thing__reference_id__reported_by_id unique(reference_id, reported_by_id)
);

create table data_type(
  id not null,
  created_at timestamptz not null default(now() at time zone 'utc'),

  constraint pk_data_type primary key(id),
  constraint ck_id        check(1 <= length(id)),
)

create table sensor(
  id           uuid         not null default(gen_random_uuid()),
  created_at   timestamptz  not null default(now() at time zone 'utc'),
  updated_at   timestamptz, 
  reference_id reference_id not null,
  part_of_id   uuid         not null,

  constraint pk_sensor                           primary key(id),
  constraint fk_part_of_id                       foreign key(part_of_id) references thing(id),
  constraint uq_sensor__reference_id__part_of_id unique(reference_id, part_of_id)
);

create table actuator(
  id           uuid         not null default(gen_random_uuid()),
  created_at   timestamptz  not null default(now() at time zone 'utc'),
  updated_at   timestamptz, 
  reference_id reference_id not null,
  part_of_id   uuid         not null,

  constraint pk_actuator   primary key(id),
  constraint fk_part_of_id foreign key(part_of_id) references thing(id),
  constraint uq_actuator__reference_id__part_of_id unique(reference_id, part_of_id)
);

create table signal_reading(
  registered_at           timestamptz not null,
  received_at             timestamptz not null default(now() at time zone 'utc'),
  registered_by_sensor_id uuid        not null
);

select create_hypertable(
  'signal_reading',
  -- TODO: Check compression settings, if data is older than compression cut-off separate backfill strategy migth be needed
  by_range('registered_at')
);
