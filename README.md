# Typst bindings for C/C++


```C
#include "typst.h"
#include <stdio.h>

int main(void) {
  TypstWorld *world = typst_world_new("../", "Hello World!");
  TypstDocument *doc = typst_world_compile(world, NULL);

  size_t len = 0;
  unsigned char *data = NULL;
  typst_document_to_pdf(doc, &len, &data, NULL);

  FILE *f = fopen("output.pdf", "wb");
  fwrite(data, 1, len, f);
  fclose(f);

  typst_pdf_free(data, len);
  typst_document_free(doc);
  typst_world_free(world);
}

```
