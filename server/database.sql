create table hub (
  id          bigserial   not null,
  created_at  timestamptz not null default (now() at time zone 'utc'),
  updated_at  timestamptz,
  external_id uuid        not null default gen_random_uuid(),
  name        text        not null,

  constraint pk_hub primary key(
    id
  ),
  constraint uq_name unique(
    name
  ),
  constraint ck_name__length check(
    length(name) <= 80
  ),
  constraint ck_name__characters check(
    name ~ '^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$'
  )
);

create table thing {
  id                   bigserial   not null,
  created_at           timestamptz not null default (now() at time zone 'utc'),
  updated_at           timestamptz,
  registered_by_hub_id bigint      not null,
  hub_reference        text        not null,

  constraint pk_thing primary key(
    id
  ),
  constraint fk_registered_by_id foreign key(
    registered_by_hub_id
  ) references hub(
    id
  ),
  constraint uq_hub_reference unique(
    registered_by_hub_id, 
    hub_reference
  ),
  constraint ck_hub_reference__length check(
    length(hub_reference) <= 80
  ),
  constraint ck_hub_reference__characters check(
    hub_reference ~ '^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$'
  )
}

create table sensor (
  id               bigserial   not null,
  created_at       timestamptz not null default (now() at time zone 'utc'),
  part_of_thing_id bigint      not null,
  hub_reference    text        not null,

  constraint pk_sensor primary key(
    id
  ),
  constraint fk_part_of_thing_id foreign key(
    part_of_thing_id
  ) references thing(
    id
  ),
  constraint uq_part_of_thing_id__hub_reference unique(
    part_of_thing_id, 
    hub_reference
  ),
  constraint ck_hub_reference__length check(
    length(hub_reference) <= 80
  ),
  constraint ck_hub_reference__characters check(
    hub_reference ~ '^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$'
  )
);

create table actuator (
  id               bigserial   not null,
  created_at       timestamptz not null default (now() at time zone 'utc'),
  part_of_thing_id bigint      not null,
  hub_reference    text        not null,

  constraint pk_actuator primary key(
    id
  ),
  constraint fk_part_of_thing_id foreign key(
    part_of_thing_id
  ) references thing(
    id
  ),
  constraint uq_part_of_thing_id__hub_reference unique(
    part_of_thing_id, 
    hub_reference
  ),
  constraint ck_hub_reference__length check(
    length(hub_reference) <= 80
  ),
  constraint ck_hub_reference__characters check(
    hub_reference ~ '^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$'
  )
);

create table boolean_reading (
  time                    timestamptz not null default (now() at time zone 'utc'),
  value                   boolean     not null,
  registered_by_sensor_id bigint      not null,
);

select create_hypertable(
  'boolean_reading', 
  by_range('time')
);

create index ix_boolean_reading__registered_by_sensor_id__time_desc ON boolean_reading (registered_by_sensor_id, time desc);

create table signal_actuation (
  time                    timestamptz not null default (now() at time zone 'utc'),
  actuated_by_actuator_id bigint      not null,
);

select create_hypertable(
  'signal_actuation', 
  by_range('time')
);

create index ix_boolean_reading__registered_by_sensor_id__time_desc ON boolean_reading (registered_by_sensor_id, time desc);

--create table actuation (
--
--);
