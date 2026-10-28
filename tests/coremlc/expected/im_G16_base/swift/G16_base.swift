//
// G16_base.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class G16_baseInput : MLFeatureProvider {

    /// img as grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 6 pixels wide by 4 pixels high
    public var img: CVPixelBuffer

    public var featureNames: Set<String> { ["img"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "img" {
            return MLFeatureValue(pixelBuffer: img)
        }
        return nil
    }

    public init(img: CVPixelBuffer) {
        self.img = img
    }

    public convenience init(imgWith img: CGImage) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!)
    }

    public convenience init(imgAt img: URL) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!)
    }

    public func setImg(with img: CGImage) throws  {
        self.img = try MLFeatureValue(cgImage: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

    public func setImg(with img: URL) throws  {
        self.img = try MLFeatureValue(imageAt: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class G16_baseOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
    public var reduce_mean_0: MLMultiArray {
        provider.featureValue(for: "reduce_mean_0")!.multiArrayValue!
    }

    /// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public var reduce_mean_0ShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(reduce_mean_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reduce_mean_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reduce_mean_0" : MLFeatureValue(multiArray: reduce_mean_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class G16_base {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "G16_base", withExtension:"mlmodelc")!
    }

    /**
        Construct G16_base instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of G16_base.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `G16_base.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct G16_base instance with explicit path to mlmodelc file
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
        Construct G16_base instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<G16_base, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct G16_base instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> G16_base {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct G16_base instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<G16_base, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(G16_base(model: model)))
            }
        }
    }

    /**
        Construct G16_base instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> G16_base {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return G16_base(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as G16_baseInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G16_baseOutput
    */
    public func prediction(input: G16_baseInput) throws -> G16_baseOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as G16_baseInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G16_baseOutput
    */
    public func prediction(input: G16_baseInput, options: MLPredictionOptions) throws -> G16_baseOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return G16_baseOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as G16_baseInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G16_baseOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: G16_baseInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> G16_baseOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return G16_baseOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 6 pixels wide by 4 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G16_baseOutput
    */
    public func prediction(img: CVPixelBuffer) throws -> G16_baseOutput {
        let input_ = G16_baseInput(img: img)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [G16_baseInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [G16_baseOutput]
    */
    public func predictions(inputs: [G16_baseInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [G16_baseOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [G16_baseOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  G16_baseOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
