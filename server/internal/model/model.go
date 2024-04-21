package model

import (
  "fmt"
	"encoding/json"
	"errors"
	"regexp"
	"time"

	"github.com/deatil/go-encoding/base62"
	"github.com/google/uuid"
)

type Id struct {
	value uuid.UUID
}

func NewId() (Id, error) {
	value, err := uuid.NewV7()
	if err != nil {
		return Id{}, err
	}

	return Id{value}, nil
}

func (i Id) MarshalJSON() ([]byte, error) {
	return json.Marshal(i.value)
}

func IdFromValue(value uuid.UUID) Id {
	return Id{value}
}

func (i Id) Value() uuid.UUID {
	return i.value
}

func (id Id) ToExternal() ExternalId {
	value := base62.StdEncoding.EncodeToString(id.value[:])
	return ExternalId{id, value}
}

type ExternalId struct {
	id    Id
	value string
}

func (e ExternalId) Id() Id {
	return e.id
}

func (e ExternalId) Value() string {
	return e.value
}

type Timestamp struct {
	value time.Time
}

func TimestampFromValue(value time.Time) Timestamp {
	return Timestamp{value}
}

func (t Timestamp) Value() time.Time {
	return t.value
}

func (t Timestamp) MarshalJSON() ([]byte, error) {
	return json.Marshal(t.value)
}

func (o Option[T]) MarshalJSON() ([]byte, error) {
	if o.IsNone() {
		return json.Marshal(nil)
	}

	return json.Marshal(o.value)
}

type Reference struct {
	value string
}

func (r Reference) MarshalJSON() ([]byte, error) {
	return json.Marshal(r.value)
}

func (r *Reference) UnmarshalJSON(data []byte) error {
  var value string 
  json.Unmarshal(data, &value)

  reference, err := ReferenceFromValue(value)
  if err != nil {
    fmt.Println(string(data))
    return err
  }

  r.value = reference.value

  return nil
}

func (r Reference) Value() string {
	return r.value
}

type CreateHub struct {
  Name Reference `json:"name"`
}

type Hub struct {
	Id        Id                `json:"id"`
	CreatedAt Timestamp         `json:"createdAt"`
	UpdatedAt Option[Timestamp] `json:"updatedAt"`
	Name      Reference         `json:"name"`
}

func ReferenceFromValue(value string) (Reference, error) {
	value_regexp, e := regexp.Compile("^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$")
	if e != nil {
		panic(e)
	}

	if !value_regexp.MatchString(value) {
		return Reference{}, errors.New("Invalid reference")
	}

	return Reference{value}, nil
}
