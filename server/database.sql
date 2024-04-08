--create table hub (
--  
--);

create table thing (
  id          bigserial   not null,
  external_id uuid        not null default gen_random_uuid(),
  created_at  timestamptz not null default (now() at time zone 'utc'),
  name        text        not null,

  constraint pk_thing primary key(
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

create unique index ix_thing__id on thing(id);

--create table sensor (
--
--);
--
--create table actuator (
--
--);
--
--create table reading (
--
--);
--
--create table actuation (
--
--);
