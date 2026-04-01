# Go Fence

```go
package preview

type Renderer struct{}

func (r *Renderer) Render() string {
    return "ok"
}
```

```golang
package preview

func NewRenderer(name string) string {
    return name
}
```
