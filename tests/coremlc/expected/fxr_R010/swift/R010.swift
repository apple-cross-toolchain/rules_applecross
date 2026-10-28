//
// R010.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class R010Input : MLFeatureProvider {

    /// in0 as 1 × 4 × 53 × 5 4-dimensional array of floats
    public var in0: MLMultiArray

    /// in1 as 6 × 5 × 1 × 4 × 3 5-dimensional array of 16-bit floats
    public var in1: MLMultiArray

    /// in2 as 4 element vector of 32-bit integers
    public var in2: MLMultiArray

    public var featureNames: Set<String> { ["in0", "in1", "in2"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "in0" {
            return MLFeatureValue(multiArray: in0)
        }
        if featureName == "in1" {
            return MLFeatureValue(multiArray: in1)
        }
        if featureName == "in2" {
            return MLFeatureValue(multiArray: in2)
        }
        return nil
    }

    public init(in0: MLMultiArray, in1: MLMultiArray, in2: MLMultiArray) {
        self.in0 = in0
        self.in1 = in1
        self.in2 = in2
    }

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public convenience init(in0: MLShapedArray<Float>, in1: MLShapedArray<Float16>, in2: MLShapedArray<Int32>) {
        self.init(in0: MLMultiArray(in0), in1: MLMultiArray(in1), in2: MLMultiArray(in2))
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class R010Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// relu_0 as multidimensional array of floats
    public var relu_0: MLMultiArray {
        provider.featureValue(for: "relu_0")!.multiArrayValue!
    }

    /// relu_0 as multidimensional array of floats
    public var relu_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(relu_0)
    }

    /// relu_1 as multidimensional array of 16-bit floats
    public var relu_1: MLMultiArray {
        provider.featureValue(for: "relu_1")!.multiArrayValue!
    }

    /// relu_1 as multidimensional array of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public var relu_1ShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(relu_1)
    }

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

    public init(relu_0: MLMultiArray, relu_1: MLMultiArray, add_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["relu_0" : MLFeatureValue(multiArray: relu_0), "relu_1" : MLFeatureValue(multiArray: relu_1), "add_0" : MLFeatureValue(multiArray: add_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class R010 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "R010", withExtension:"mlmodelc")!
    }

    /**
        Construct R010 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of R010.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `R010.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct R010 instance with explicit path to mlmodelc file
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
        Construct R010 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<R010, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct R010 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> R010 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct R010 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<R010, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(R010(model: model)))
            }
        }
    }

    /**
        Construct R010 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> R010 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return R010(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as R010Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R010Output
    */
    public func prediction(input: R010Input) throws -> R010Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as R010Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R010Output
    */
    public func prediction(input: R010Input, options: MLPredictionOptions) throws -> R010Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return R010Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as R010Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R010Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: R010Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> R010Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return R010Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - in0: 1 × 4 × 53 × 5 4-dimensional array of floats
            - in1: 6 × 5 × 1 × 4 × 3 5-dimensional array of 16-bit floats
            - in2: 4 element vector of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R010Output
    */
    public func prediction(in0: MLMultiArray, in1: MLMultiArray, in2: MLMultiArray) throws -> R010Output {
        let input_ = R010Input(in0: in0, in1: in1, in2: in2)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - in0: 1 × 4 × 53 × 5 4-dimensional array of floats
            - in1: 6 × 5 × 1 × 4 × 3 5-dimensional array of 16-bit floats
            - in2: 4 element vector of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R010Output
    */

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public func prediction(in0: MLShapedArray<Float>, in1: MLShapedArray<Float16>, in2: MLShapedArray<Int32>) throws -> R010Output {
        let input_ = R010Input(in0: in0, in1: in1, in2: in2)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [R010Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [R010Output]
    */
    public func predictions(inputs: [R010Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [R010Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [R010Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  R010Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
