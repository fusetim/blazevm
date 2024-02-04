public class ObjectTest {

    public static final ObjectTest INSTANCE = new ObjectTest();

    int count = 0;

    public static void main() {
        int a = ObjectTest.INSTANCE.get();
        ObjectTest.INSTANCE.incr();
        int b = ObjectTest.INSTANCE.get();
        ObjectTest.INSTANCE.incr();
        int c = ObjectTest.INSTANCE.get();
    }

    public int get() {
        return this.count;
    }

    public void incr() {
        this.count++;
    }
}