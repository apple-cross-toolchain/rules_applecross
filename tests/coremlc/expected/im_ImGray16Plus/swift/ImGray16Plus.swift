//
// ImGray16Plus.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImGray16PlusInput : MLFeatureProvider {

    /// g as grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
    public var g: CVPixelBuffer

    /// t as 1 by 4 matrix of floats
    public var t: MLMultiArray

    public var featureNames: Set<String> { ["g", "t"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "g" {
            return MLFeatureValue(pixelBuffer: g)
        }
        if featureName == "t" {
            return MLFeatureValue(multiArray: t)
        }
        return nil
    }

    public init(g: CVPixelBuffer, t: MLMultiArray) {
        self.g = g
        self.t = t
    }

    public convenience init(g: CVPixelBuffer, t: MLShapedArray<Float>) {
        self.init(g: g, t: MLMultiArray(t))
    }

    public convenience init(gWith g: CGImage, t: MLMultiArray) throws {
        self.init(g: try MLFeatureValue(cgImage: g, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: t)
    }

    public convenience init(gWith g: CGImage, t: MLShapedArray<Float>) throws {
        self.init(g: try MLFeatureValue(cgImage: g, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: MLMultiArray(t))
    }

    public convenience init(gAt g: URL, t: MLMultiArray) throws {
        self.init(g: try MLFeatureValue(imageAt: g, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: t)
    }

    public convenience init(gAt g: URL, t: MLShapedArray<Float>) throws {
        self.init(g: try MLFeatureValue(imageAt: g, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: MLMultiArray(t))
    }

    public func setG(with g: CGImage) throws  {
        self.g = try MLFeatureValue(cgImage: g, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

    public func setG(with g: URL) throws  {
        self.g = try MLFeatureValue(imageAt: g, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImGray16PlusOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reshape_0 as 1 element vector of floats
    public var reshape_0: MLMultiArray {
        provider.featureValue(for: "reshape_0")!.multiArrayValue!
    }

    /// reshape_0 as 1 element vector of floats
    public var reshape_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reshape_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reshape_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reshape_0" : MLFeatureValue(multiArray: reshape_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImGray16Plus {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ImGray16Plus", withExtension:"mlmodelc")!
    }

    /**
        Construct ImGray16Plus instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ImGray16Plus.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ImGray16Plus.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct ImGray16Plus instance with explicit path to mlmodelc file
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
        Construct ImGray16Plus instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImGray16Plus, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ImGray16Plus instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImGray16Plus {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ImGray16Plus instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImGray16Plus, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ImGray16Plus(model: model)))
            }
        }
    }

    /**
        Construct ImGray16Plus instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImGray16Plus {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ImGray16Plus(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImGray16PlusInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImGray16PlusOutput
    */
    public func prediction(input: ImGray16PlusInput) throws -> ImGray16PlusOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImGray16PlusInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImGray16PlusOutput
    */
    public func prediction(input: ImGray16PlusInput, options: MLPredictionOptions) throws -> ImGray16PlusOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ImGray16PlusOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImGray16PlusInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImGray16PlusOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ImGray16PlusInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ImGray16PlusOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ImGray16PlusOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - g: grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
            - t: 1 by 4 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImGray16PlusOutput
    */
    public func prediction(g: CVPixelBuffer, t: MLMultiArray) throws -> ImGray16PlusOutput {
        let input_ = ImGray16PlusInput(g: g, t: t)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - g: grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
            - t: 1 by 4 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImGray16PlusOutput
    */

    public func prediction(g: CVPixelBuffer, t: MLShapedArray<Float>) throws -> ImGray16PlusOutput {
        let input_ = ImGray16PlusInput(g: g, t: t)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ImGray16PlusInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ImGray16PlusOutput]
    */
    public func predictions(inputs: [ImGray16PlusInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ImGray16PlusOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ImGray16PlusOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ImGray16PlusOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
