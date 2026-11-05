//
//  Author+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias AuthorCoreDataPropertiesSet = NSSet

extension Author {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<Author> {
        return NSFetchRequest<Author>(entityName: "Author")
    }

    @NSManaged nonisolated public var name: String?
    @NSManaged nonisolated public var books: NSSet?
    @NSManaged nonisolated public var ordered: NSOrderedSet?
    @NSManaged nonisolated public var fav: Book?

}

// MARK: Generated accessors for books
extension Author {

    @objc(addBooksObject:)
    @NSManaged nonisolated public func addToBooks(_ value: Book)

    @objc(removeBooksObject:)
    @NSManaged nonisolated public func removeFromBooks(_ value: Book)

    @objc(addBooks:)
    @NSManaged nonisolated public func addToBooks(_ values: NSSet)

    @objc(removeBooks:)
    @NSManaged nonisolated public func removeFromBooks(_ values: NSSet)

}

// MARK: Generated accessors for ordered
extension Author {

    @objc(insertObject:inOrderedAtIndex:)
    @NSManaged nonisolated public func insertIntoOrdered(_ value: Book, at idx: Int)

    @objc(removeObjectFromOrderedAtIndex:)
    @NSManaged nonisolated public func removeFromOrdered(at idx: Int)

    @objc(insertOrdered:atIndexes:)
    @NSManaged nonisolated public func insertIntoOrdered(_ values: [Book], at indexes: NSIndexSet)

    @objc(removeOrderedAtIndexes:)
    @NSManaged nonisolated public func removeFromOrdered(at indexes: NSIndexSet)

    @objc(replaceObjectInOrderedAtIndex:withObject:)
    @NSManaged nonisolated public func replaceOrdered(at idx: Int, with value: Book)

    @objc(replaceOrderedAtIndexes:withOrdered:)
    @NSManaged nonisolated public func replaceOrdered(at indexes: NSIndexSet, with values: [Book])

    @objc(addOrderedObject:)
    @NSManaged nonisolated public func addToOrdered(_ value: Book)

    @objc(removeOrderedObject:)
    @NSManaged nonisolated public func removeFromOrdered(_ value: Book)

    @objc(addOrdered:)
    @NSManaged nonisolated public func addToOrdered(_ values: NSOrderedSet)

    @objc(removeOrdered:)
    @NSManaged nonisolated public func removeFromOrdered(_ values: NSOrderedSet)

}

extension Author : Identifiable {

}
