#include <iostream>
#include <vector>

int main() {
    std::vector<int> numbers = {1, 2, 3, 4, 5};
    int sum = 0;
    for (int n : numbers) {
        sum += n * n;
    }
    std::cout << "Sum of squares: " << sum << std::endl;
    return 0;
}
