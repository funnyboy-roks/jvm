public class Function {
    public static int add(int a, int b) {
        int x = a + b;
        return x;
    }

    public static int sub(int a, int b) {
        int x = a - b;
        return x;
    }

    public static void main(String... args) {
        int y = add(add(1, 2), 3);
        int x = add(sub(4, 5), 6);
    }
}
