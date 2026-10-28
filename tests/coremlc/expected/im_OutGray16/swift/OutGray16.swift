//
// OutGray16.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutGray16Input : MLFeatureProvider {

    /// x as 1 × 1 × 8 × 8 4-dimensional array of 16-bit floats
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

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public convenience init(x: MLShapedArray<Float16>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutGray16Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// image as grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
    public var image: CVPixelBuffer {
        provider.featureValue(for: "image")!.imageBufferValue!
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(image: CVPixelBuffer) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["image" : MLFeatureValue(pixelBuffer: image)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutGray16 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "OutGray16", withExtension:"mlmodelc")!
    }

    /**
        Construct OutGray16 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of OutGray16.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `OutGray16.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct OutGray16 instance with explicit path to mlmodelc file
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
        Construct OutGray16 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<OutGray16, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct OutGray16 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> OutGray16 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct OutGray16 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<OutGray16, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(OutGray16(model: model)))
            }
        }
    }

    /**
        Construct OutGray16 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> OutGray16 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return OutGray16(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutGray16Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutGray16Output
    */
    public func prediction(input: OutGray16Input) throws -> OutGray16Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutGray16Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutGray16Output
    */
    public func prediction(input: OutGray16Input, options: MLPredictionOptions) throws -> OutGray16Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return OutGray16Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutGray16Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutGray16Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: OutGray16Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> OutGray16Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return OutGray16Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 1 × 8 × 8 4-dimensional array of 16-bit floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutGray16Output
    */
    public func prediction(x: MLMultiArray) throws -> OutGray16Output {
        let input_ = OutGray16Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 1 × 8 × 8 4-dimensional array of 16-bit floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutGray16Output
    */

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public func prediction(x: MLShapedArray<Float16>) throws -> OutGray16Output {
        let input_ = OutGray16Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [OutGray16Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [OutGray16Output]
    */
    public func predictions(inputs: [OutGray16Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [OutGray16Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [OutGray16Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  OutGray16Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
