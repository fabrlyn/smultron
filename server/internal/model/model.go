package model

import (
	"database/sql"
	"errors"
	"regexp"
	"time"

	"github.com/deatil/go-encoding/base62"
	"github.com/google/uuid"
)

type Id struct {
	value uuid.UUID
}

type ExternalId struct {
	Id    Id
	Value string
}

type Reference struct {
	value string
}

func NewId() (Id, error) {
	value, err := uuid.NewV7()
	if err != nil {
		return Id{}, err
	}

	return Id{value}, nil
}

func (id *Id) ToExternal() ExternalId {
	base62.StdEncoding.EncodeToString(id.value[:])
	return ExternalId{}
}

type CreateHub struct {
	name Reference
}

type Hub struct {
	id         Id
	created_at time.Time    // TODO: Timestamp
	updated_at sql.NullTime // TODO: Maybe[Timestamp]
	name       Reference
}

func newReference(value string) (Reference, error) {
	value_regexp, e := regexp.Compile("^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$")
	if e != nil {
		panic(e)
	}

	if !value_regexp.MatchString(value) {
		return Reference{}, errors.New("Invalid reference")
	}

	return Reference{value}, nil
}
