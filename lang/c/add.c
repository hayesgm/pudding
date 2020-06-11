#include <stdbool.h>

int __attribute__ ((noinline)) square(int a) {
  return a*a;
}

int add(int a, int b) {
  return square(a) + b + 5;
}

// bool size(int a) {
//   if (a > 10) {
//     return true;
//   } else {
//     return false;
//   }
// }

// int loop(int a) {
//   int res = 0;
//   for (int i = 0; i < a; i++) {
//     res += 3;
//   }
//   return res;
// }

// int loopy(int a) {
//   int res = 0;
//   while (res * res < a) {
//     res++;
//   }
//   return res;
// }

// int locals(int a) {
//   int b = a;
//   b += 1;
//   return b;
// }

int g = 0;
int globals() {
  return ++g;
}

// int sum(int a) {
//   int res = 0;
//   for (int i = 0; i < a; i++) {
//     res += i;
//   }
//   return res;
// }
