# Go Fence Showcase

```go
package showcase

type Renderer[T comparable] struct {
    Name string
}

func (r *Renderer[T]) Render() string {
    return "ok"
}
```

```golang
package showcase

func NewRenderer(name string) string {
    return name
}
```
