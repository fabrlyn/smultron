--create table hub (
--  
--);

create table thing (
  internal_id bigserial   not null,
  created_at  timestamptz not null default (now() at time zone 'utc'),
  id          text        not null,

  constraint thing_pk primary key(
    internal_id
  ),
  constraint ck_id__length check(
    length(id) <= 63
  ),
  constraint ck_id__characters check(
    id ~ '^[a-z]+(-[a-z0-9]+)?$'
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
