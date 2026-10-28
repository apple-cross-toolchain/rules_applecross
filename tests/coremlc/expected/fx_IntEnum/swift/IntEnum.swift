//
// IntEnum.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class IntEnumInput : MLFeatureProvider {

    /// n as 1 by 4 matrix of 32-bit integers
    public var n: MLMultiArray

    public var featureNames: Set<String> { ["n"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "n" {
            return MLFeatureValue(multiArray: n)
        }
        return nil
    }

    public init(n: MLMultiArray) {
        self.n = n
    }

    public convenience init(n: MLShapedArray<Int32>) {
        self.init(n: MLMultiArray(n))
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class IntEnumOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// add_0 as multidimensional array of 32-bit integers
    public var add_0: MLMultiArray {
        provider.featureValue(for: "add_0")!.multiArrayValue!
    }

    /// add_0 as multidimensional array of 32-bit integers
    public var add_0ShapedArray: MLShapedArray<Int32> {
        MLShapedArray<Int32>(add_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(add_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["add_0" : MLFeatureValue(multiArray: add_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class IntEnum {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "IntEnum", withExtension:"mlmodelc")!
    }

    /**
        Construct IntEnum instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of IntEnum.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `IntEnum.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct IntEnum instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct IntEnum instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<IntEnum, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct IntEnum instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> IntEnum {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct IntEnum instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<IntEnum, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(IntEnum(model: model)))
            }
        }
    }

    /**
        Construct IntEnum instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> IntEnum {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return IntEnum(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as IntEnumInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IntEnumOutput
    */
    public func prediction(input: IntEnumInput) throws -> IntEnumOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as IntEnumInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IntEnumOutput
    */
    public func prediction(input: IntEnumInput, options: MLPredictionOptions) throws -> IntEnumOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return IntEnumOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as IntEnumInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IntEnumOutput
    */
    public func prediction(input: IntEnumInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> IntEnumOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return IntEnumOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - n: 1 by 4 matrix of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IntEnumOutput
    */
    public func prediction(n: MLMultiArray) throws -> IntEnumOutput {
        let input_ = IntEnumInput(n: n)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - n: 1 by 4 matrix of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IntEnumOutput
    */

    public func prediction(n: MLShapedArray<Int32>) throws -> IntEnumOutput {
        let input_ = IntEnumInput(n: n)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [IntEnumInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [IntEnumOutput]
    */
    public func predictions(inputs: [IntEnumInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [IntEnumOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [IntEnumOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  IntEnumOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
