#include <iostream>

class Animal {
public:
    int age;
    virtual void speak() {
        std::cout << "I am the generic 'Animal'. I am an amorphous blob that should not be able to speak, yet here we are\n";
    }
    int get_age() { return age; }
};

class Dog: public Animal {
public:
    Dog(int age) {
        age = age;
    }
    void speak() {
        std::cout << "WOOF\n";
    }
    int get_age() { return age * 7; }
};

class ThinDog {
    int age;
public:
    ThinDog(int age) {
        age = age;
    }
    void speak() {
        std::cout << "wof\n";
    }
    int get_age() {
        return age * 7;
    }
};

int main() {
    Dog dog(2);
    dog.speak();
    ((Animal)dog).speak();
    ThinDog thinDog(3);
    thinDog.speak();
    std::cout << "\n";

    std::cout << "sizeof(Dog) = " << sizeof(dog) << "\n";
    std::cout << "sizeof(ThinDog) = " << sizeof(thinDog) << "\n\n";

    Dog silentDog(4);
    std::cout << "How old is silent dog? - " << silentDog.get_age() << "\n";
    std::cout << "sizeof(silentDog) = " << sizeof(silentDog) << "\n";
}