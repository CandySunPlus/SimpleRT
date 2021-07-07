#ifndef __GLIBC__
#ifndef RESOLV_COMPAT_H

#define RESOLV_COMPAT_H

#include <resolv.h>
#include <string.h>

static inline int res_ninit(res_state statp) {
  int rc = res_init();
  if (statp != &_res) {
    memcpy(statp, &_res, sizeof(*statp));
  }
  return rc;
}

static inline int res_nclose(res_state statp) {
  if (!statp)
    return -1;

  if (statp != &_res) {
    memset(statp, 0, sizeof(*statp));
  }
  return 0;
}

#endif /* end of include guard: RESOLV_COMPAT_H */
#endif
