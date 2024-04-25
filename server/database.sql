begin;

create function validate_reference_characters(text) returns boolean as
$$ 
begin
  return $1 ~ '^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$'; 
end 
$$ language plpgsql;

create function validate_reference_length(text) returns boolean as
$$
begin
  return length($1) <= 80;
end
$$ language plpgsql;

create function validate_reference(text) returns boolean as
$$
begin
  return validate_reference_characters($1) and validate_reference_length($1);
end
$$ language plpgsql;

create table hub(
  id          uuid        not null,
  created_at  timestamptz not null default (now() at time zone 'utc'),
  updated_at  timestamptz,
  name        text        not null,

  constraint pk_hub       primary key(id),
  constraint ck_name      check(validate_reference(name)),
  constraint uq_hub__name unique(name)
);

create table thing(
  id                   uuid        not null,
  created_at           timestamptz not null default (now() at time zone 'utc'),
  registered_by_hub_id uuid        not null,
  hub_reference        text        not null,

  constraint pk_thing                primary key(id),
  constraint uq_thing__hub_reference unique(registered_by_hub_id, hub_reference),
  constraint ck_hub_reference        check(validate_reference(hub_reference)),
  constraint fk_registered_by_id     foreign key(registered_by_hub_id) references hub(id)
);

create table sensor(
  id               uuid        not null,
  created_at       timestamptz not null default (now() at time zone 'utc'),
  part_of_thing_id uuid        not null,
  hub_reference    text        not null,

  constraint pk_sensor                                  primary key(id),
  constraint uq_sensor__part_of_thing_id__hub_reference unique(part_of_thing_id, hub_reference),
  constraint ck_hub_reference                           check(validate_reference(hub_reference)),
  constraint fk_part_of_thing_id                        foreign key(part_of_thing_id) references thing(id)
);

create table actuator(
  id               uuid        not null,
  created_at       timestamptz not null default (now() at time zone 'utc'),
  part_of_thing_id uuid        not null,
  hub_reference    text        not null,

  constraint pk_actuator                        primary key(id),
  constraint uq_part_of_thing_id__hub_reference unique(part_of_thing_id, hub_reference),
  constraint ck_hub_reference                   check(validate_reference(hub_reference)),
  constraint fk_part_of_thing_id                foreign key(part_of_thing_id) references thing(id)
);

-- boolean reading

create table boolean_reading(
  registered_at           timestamptz not null,
  received_at             timestamptz not null default (now() at time zone 'utc'),
  value                   boolean     not null,
  registered_by_sensor_id uuid        not null
);

select create_hypertable(
  'boolean_reading', 
  by_range('received_at')
);

create 
  index ix_boolean_reading__registered_by_sensor_id__registered_at_desc 
  on boolean_reading(
    registered_by_sensor_id, 
    registered_at desc
  );

-- i32 reading

create table i32_reading(
  registered_at           timestamptz not null,
  received_at             timestamptz not null default (now() at time zone 'utc'),
  value                   int         not null,
  registered_by_sensor_id uuid        not null
);

select create_hypertable(
  'i32_reading', 
  by_range('received_at')
);

create 
  index ix_i32_reading__registered_by_sensor_id__registered_at_desc 
  on boolean_reading(
    registered_by_sensor_id, 
    registered_at desc
  );

-- signal actuation

create table signal_actuation(
  time                    timestamptz not null default (now() at time zone 'utc'),
  actuated_by_actuator_id uuid        not null
);

select create_hypertable(
  'signal_actuation', 
  by_range('time')
);

create 
  index ix_signal_actuation__registered_by_sensor_id__time_desc 
  on signal_actuation(
    actuated_by_actuator_id, 
    time desc
  );

commit;

