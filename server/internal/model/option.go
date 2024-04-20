package model

type Option[T any] struct {
  hasValue bool
  value T
}

func NewOption[T any](value *T) Option[T] {
  if value == nil {
    return None[T]()
  }

  return Some(*value)
}

func (o Option[T]) IsSome() bool {
  return o.hasValue
}

func (o Option[T]) IsNone() bool {
  return !o.IsSome()
}

func Map[T any, U any](option Option[T], fn func(value T) U) Option[U] {
  if option.IsNone() {
    return None[U]()
  }
  return Some(fn(option.value))
}

func MapToPointer[T any](option Option[T]) Option[*T] {
  return Map(option, func(value T) *T {
    return &value
  })
}

func (o Option[T]) GetOr(other T) T {
  if o.IsNone() {
    return other
  }

  return o.value
}

func (o Option[T]) GetOrElse(f func() T) T {
  if o.IsNone() {
    return f()
  }

  return o.value
}

func Some[T any](value T) Option[T] {
  return Option[T] {
    hasValue:  true,
    value: value,
  }
}

func None[T any]() Option[T] {
  return Option[T] {
    hasValue: false,
    value: *new(T),
  }
}

