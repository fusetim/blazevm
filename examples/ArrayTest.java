public class ArrayTest {
    public static int sum2(int n) {
        int[] arr = new int[n];
        for (int i = 0; i < n; i++) {
            arr[i] = i;
        }
        int sum2 = 0;
        for (int i = 0; i < n; i += 2) {
            sum2 += arr[i];
        }
        return sum2;
    }

    /// Create a 2D array of size n x m.
    ///
    /// Currently necessary as BlazeVM does not support `multianewarray` opcode.
    public static double[][] matrix(int n, int m) {
        double[][] arr = new double[n][];
        for (int i = 0; i < n; i++) {
            arr[i] = new double[m];
        }
        return arr;
    }

    public static double[][] diag(double[] d) {
        int n = d.length;
        double[][] arr = matrix(n,n);
        for (int i = 0; i < n; i++) {
            arr[i][i] = d[i];
        }
        return arr;
    }

    public static double tr(double[][] arr) {
        int n = arr.length;
        double sum = 0;
        for (int i = 0; i < n; i++) {
            sum += arr[i][i];
        }
        return sum;
    }

    public static void main() {
        int rst = sum2(10);
        // Expected result: 20
        double[] d = { 1, 2, 3, 4 };
        double[][] arr = diag(d);
        double tr = tr(arr);
        // Expected result: 10.0
    }
}
