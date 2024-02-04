public class Fibonacci {
    public static int fibonacci(int n) {
        if (n <= 1) {
            return n;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }

    public static int imp_fibonacci(int n) {
        int x = 0;
        int y = 1;
        for (int i = 0; i < n; i++) {
            int temp = x;
            x = y;
            y = temp + y;
        }
        return x;
    }

    public static void main() {
        int rst = imp_fibonacci(40);
    }
}
