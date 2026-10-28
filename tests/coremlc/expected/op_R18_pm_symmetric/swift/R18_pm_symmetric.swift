//
// R18_pm_symmetric.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class R18_pm_symmetricInput : MLFeatureProvider {

    /// x as 1 × 2 × 4 × 4 4-dimensional array of floats
    public var x: MLMultiArray

    public var featureNames: Set<String> { ["x"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        return nil
    }

    public init(x: MLMultiArray) {
        self.x = x
    }

    public convenience init(x: MLShapedArray<Float>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class R18_pm_symmetricOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// resample_0 as 1 × 2 × 3 × 3 4-dimensional array of floats
    public var resample_0: MLMultiArray {
        provider.featureValue(for: "resample_0")!.multiArrayValue!
    }

    /// resample_0 as 1 × 2 × 3 × 3 4-dimensional array of floats
    public var resample_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(resample_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(resample_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["resample_0" : MLFeatureValue(multiArray: resample_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class R18_pm_symmetric {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "R18_pm_symmetric", withExtension:"mlmodelc")!
    }

    /**
        Construct R18_pm_symmetric instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of R18_pm_symmetric.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `R18_pm_symmetric.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct R18_pm_symmetric instance with explicit path to mlmodelc file
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
        Construct R18_pm_symmetric instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<R18_pm_symmetric, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct R18_pm_symmetric instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> R18_pm_symmetric {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct R18_pm_symmetric instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<R18_pm_symmetric, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(R18_pm_symmetric(model: model)))
            }
        }
    }

    /**
        Construct R18_pm_symmetric instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> R18_pm_symmetric {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return R18_pm_symmetric(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as R18_pm_symmetricInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R18_pm_symmetricOutput
    */
    public func prediction(input: R18_pm_symmetricInput) throws -> R18_pm_symmetricOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as R18_pm_symmetricInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R18_pm_symmetricOutput
    */
    public func prediction(input: R18_pm_symmetricInput, options: MLPredictionOptions) throws -> R18_pm_symmetricOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return R18_pm_symmetricOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as R18_pm_symmetricInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R18_pm_symmetricOutput
    */
    public func prediction(input: R18_pm_symmetricInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> R18_pm_symmetricOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return R18_pm_symmetricOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 2 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R18_pm_symmetricOutput
    */
    public func prediction(x: MLMultiArray) throws -> R18_pm_symmetricOutput {
        let input_ = R18_pm_symmetricInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 2 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as R18_pm_symmetricOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> R18_pm_symmetricOutput {
        let input_ = R18_pm_symmetricInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [R18_pm_symmetricInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [R18_pm_symmetricOutput]
    */
    public func predictions(inputs: [R18_pm_symmetricInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [R18_pm_symmetricOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [R18_pm_symmetricOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  R18_pm_symmetricOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
