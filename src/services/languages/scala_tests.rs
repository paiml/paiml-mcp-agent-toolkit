#[cfg(all(test, feature = "scala-ast"))]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_SCALA_CLASS: &str = r#"
    package com.example

    class HelloWorld {
      def sayHello(): String = {
        "Hello, World!"
      }
    }
    "#;

    const SCALA_TRAIT_EXAMPLE: &str = r#"
    package com.example.shapes

    trait Shape {
      def area(): Double
      def perimeter(): Double
    }

    class Circle(radius: Double) extends Shape {
      def area(): Double = math.Pi * radius * radius
      def perimeter(): Double = 2 * math.Pi * radius
    }
    "#;

    const SCALA_CASE_CLASS_EXAMPLE: &str = r#"
    package com.example.models

    case class Person(name: String, age: Int) {
      def isAdult: Boolean = age >= 18
    }

    object Person {
      def apply(name: String): Person = new Person(name, 0)
    }
    "#;

    const SCALA_COMPREHENSIVE_EXAMPLE: &str = r#"
    package com.example.functional

    import scala.concurrent.Future
    import scala.concurrent.ExecutionContext.Implicits.global

    // A trait defining functional operations
    trait Functor[F[_]] {
      def map[A, B](fa: F[A])(f: A => B): F[B]
    }

    // A case class for user data
    case class User(id: String, name: String, email: String)

    // An object with utility methods
    object UserService {
      private val users = Map(
        "1" -> User("1", "Alice", "alice@example.com"),
        "2" -> User("2", "Bob", "bob@example.com")
      )

      def getUser(id: String): Option[User] = users.get(id)

      def findUserByEmail(email: String): Option[User] =
        users.values.find(_.email == email)

      def getUserAsync(id: String): Future[Option[User]] =
        Future.successful(getUser(id))

      // Pattern matching example
      def processUserResult(result: Option[User]): String = result match {
        case Some(user) if user.name.startsWith("A") => s"A user: ${user.name}"
        case Some(user) => s"User: ${user.name}"
        case None => "User not found"
      }
    }

    // A class that uses higher-order functions
    class DataProcessor[T](data: List[T]) {
      def map[B](f: T => B): List[B] = data.map(f)

      def filter(p: T => Boolean): List[T] = data.filter(p)

      def fold[B](z: B)(op: (B, T) => B): B = data.foldLeft(z)(op)
    }
    "#;

    #[test]
    fn test_simple_scala_class_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("HelloWorld.scala"));
        let items = visitor
            .analyze_scala_source(SIMPLE_SCALA_CLASS)
            .expect("Should parse Scala class");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct {
            name, visibility, ..
        } = &class_items[0]
        {
            assert_eq!(
                name, "com.example::HelloWorld",
                "Should have qualified class name"
            );
            assert_eq!(
                visibility, "public",
                "Scala classes have public visibility by default"
            );
        } else {
            panic!("Expected class item");
        }
    }

    #[test]
    fn test_scala_trait_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("Shape.scala"));
        let items = visitor
            .analyze_scala_source(SCALA_TRAIT_EXAMPLE)
            .expect("Should parse Scala trait");

        // Check that we found the trait
        let trait_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .collect();

        assert_eq!(trait_items.len(), 1, "Should extract exactly one trait");

        if let AstItem::Trait { name, .. } = &trait_items[0] {
            assert_eq!(
                name, "com.example.shapes::Shape",
                "Should have qualified trait name"
            );
        }

        // Check that we found the implementing class
        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        // Check for methods
        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert!(
            method_items.len() >= 2,
            "Should extract at least two methods"
        );
    }

    #[test]
    fn test_scala_case_class_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("Person.scala"));
        let items = visitor
            .analyze_scala_source(SCALA_CASE_CLASS_EXAMPLE)
            .expect("Should parse Scala case class");

        // Check that we found the case class
        let case_class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert!(
            !case_class_items.is_empty(),
            "Should extract at least one case class"
        );

        let has_case_class = case_class_items.iter().any(|item| {
            if let AstItem::Struct { derives, .. } = item {
                derives.contains(&"case".to_string())
            } else {
                false
            }
        });

        assert!(has_case_class, "Should identify a case class");

        // Check that we found the companion object
        let object_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Module { .. }))
            .collect();

        assert!(
            !object_items.is_empty(),
            "Should extract at least one object"
        );
    }

    #[test]
    fn test_scala_comprehensive_analysis() {
        let visitor = ScalaAstVisitor::new(Path::new("Comprehensive.scala"));
        let items = visitor
            .analyze_scala_source(SCALA_COMPREHENSIVE_EXAMPLE)
            .expect("Should parse comprehensive Scala example");

        // Should have extracted a good number of items
        assert!(
            items.len() >= 10,
            "Should extract numerous AST items from comprehensive example"
        );

        // Check for traits, case classes, objects, and methods
        let trait_count = items
            .iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .count();

        let case_class_count = items.iter()
            .filter(|item| matches!(item, AstItem::Struct { derives, .. } if derives.contains(&"case".to_string())))
            .count();

        let object_count = items
            .iter()
            .filter(|item| matches!(item, AstItem::Module { .. }))
            .count();

        let method_count = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .count();

        assert!(trait_count > 0, "Should find at least one trait");
        assert!(case_class_count > 0, "Should find at least one case class");
        assert!(object_count > 0, "Should find at least one object");
        assert!(method_count > 0, "Should find at least one method");
    }

    #[test]
    fn test_scala_complexity_analysis() {
        let mut analyzer = ScalaComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SCALA_COMPREHENSIVE_EXAMPLE)
            .expect("Should analyze Scala complexity");

        assert!(
            cyclomatic >= 1,
            "Should have at least cyclomatic complexity of 1"
        );
        assert!(
            cognitive >= 1,
            "Should have at least cognitive complexity of 1"
        );
        assert!(
            cyclomatic <= 20,
            "Should maintain reasonable cyclomatic complexity"
        );
        assert!(
            cognitive <= 20,
            "Should maintain reasonable cognitive complexity"
        );
    }

    #[test]
    fn test_empty_scala_source() {
        let visitor = ScalaAstVisitor::new(Path::new("empty.scala"));
        let items = visitor
            .analyze_scala_source("")
            .expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_scala_syntax() {
        let visitor = ScalaAstVisitor::new(Path::new("invalid.scala"));
        let result = visitor.analyze_scala_source("invalid scala syntax {{{ !!!");

        assert!(
            result.is_err(),
            "Should return error for invalid Scala syntax"
        );
    }
}
