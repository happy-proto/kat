# Jam build files

```jam
rule greet ( name ) { ECHO "Hello $(name)" ; }
greet reader ;
```

```bjam
lib example : example.cpp : <link>shared ;
```

```boost-build
import feature ;
```
